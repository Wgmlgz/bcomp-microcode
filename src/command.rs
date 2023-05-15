use crate::regexes::*;
use crate::{
    control_signals::{
        Addr,
        BaseCs::{self},
        Bit, Comp,
        Cs::{self},
    },
    table::Table,
};
use enumset::{enum_set, EnumSet};
use fancy_regex::Match;
use itertools::Itertools;
use num_traits::{FromPrimitive, Zero};
use parse_int::parse;
use std::iter::{once, once_with};
use std::{error::Error, ops::Range, str::FromStr};
pub type ParseErr = Box<dyn Error>;
pub type ParseRes = Result<(), ParseErr>;
use bit_iter::*;

#[derive(Default, Clone, Copy)]
pub struct Command {
    pub cmd: u64,
}

pub struct Omc {
    pub base: EnumSet<BaseCs>,
}

pub trait ToCss {
    fn to_css(&self) -> Vec<Cs>;
}

impl ToCss for Omc {
    fn to_css(&self) -> Vec<Cs> {
        let mut v = vec![];
        self.base.iter().for_each(|x| v.push(Cs::Base(x)));
        v
    }
}

pub struct Cmc {
    pub base: EnumSet<BaseCs>,
    pub bit: Bit,
    pub addr: Addr,
    pub comp: Comp,
}

impl ToCss for Cmc {
    fn to_css(&self) -> Vec<Cs> {
        let mut v = vec![];
        self.base.iter().for_each(|x| v.push(Cs::Base(x)));
        v.push(Cs::Bit(self.bit.0));
        v.push(Cs::Addr(self.addr.0));
        v.push(Cs::Comp(self.comp.0));
        v
    }
}

pub enum Mc {
    Operational(Omc),
    Control(Cmc),
}
impl ToCss for Mc {
    fn to_css(&self) -> Vec<Cs> {
        match self {
            Mc::Operational(omc) => omc.to_css(),
            Mc::Control(cmc) => cmc.to_css(),
        }
    }
}

impl From<Command> for Mc {
    fn from(cmd: Command) -> Self {
        let mc_type = !(cmd.cmd & (1 << 39)).is_zero();
        let base: EnumSet<BaseCs> = (if mc_type { 0..16 } else { 0..39 })
            .filter_map(|bit| {
                (!(cmd.cmd & (1 << bit)).is_zero()).then(|| BaseCs::from_i32(bit).unwrap())
            })
            .collect();
        return if mc_type {
            Self::Control(Cmc {
                base,
                bit: (Bit(((cmd.cmd >> 16) & 0xff) as u8)),
                addr: (Addr(((cmd.cmd >> 24) & 0xff) as u8)),
                comp: (Comp(((cmd.cmd >> 32) & 0x1) != 0)),
            })
        } else {
            Self::Operational(Omc { base })
        };
    }
}

// struct Sus {
//     aboba: Cs::Base,
// }
impl Command {
    pub fn new(cmd: u64) -> Self {
        Self { cmd }
    }

    pub fn cs(&self) -> Vec<Cs> {
        let mc_type = !(self.cmd & (1 << 39)).is_zero();

        let mc: Mc = Into::<Mc>::into(*self);
        let mut res = mc.to_css();
        res.push(Cs::Type(mc_type));

        res
    }
}

#[derive(Default)]
pub struct ParseState {
    pub cmd: u64,
    pub range: Range<usize>,
}

impl ParseState {
    fn add_state(&mut self, cs: Cs) {
        match cs {
            Cs::Base(base) => base.update_state(&mut self.cmd),
            Cs::Bit(bit) => self.cmd |= (bit as u64) << 16,
            Cs::Addr(addr) => self.cmd |= (addr as u64) << 24,
            Cs::Comp(comp) => self.cmd |= (comp as u64) << 32, // fuck МЕТОДИЧКА
            Cs::Type(mc_type) => self.cmd |= (mc_type as u64) << 39,
        }
    }

    fn parse_general(&mut self, s: &str) -> ParseRes {
        dbg!("enter general");
        // just hint's for better ux
        if let Some(cap) = IF_TYPE_RE.captures(s)? {
            self.add_state(Cs::Type(true));

            let cap = IF_STRUCTURE_RE
                .captures(cap.get(2).unwrap().as_str())
                .unwrap()
                .ok_or("seems like if command, but unable to parce it")?;

            return self.parse_if(cap.get(2).unwrap(), cap.get(3).unwrap());
        } else if let Some(cap) = GOTO_TYPE_RE.captures(s)? {
            // that's how unconditional jumps works lol
            self.add_state(Cs::Type(true));
            self.add_state(Cs::Base(BaseCs::RDPS));
            self.add_state(Cs::Base(BaseCs::LTOL));
            self.add_state(Cs::Bit(0b0010000));
            return self.parse_goto(cap.get(1).unwrap());
        } else {
            return self.parse_omc(s);
        }
    }

    fn parse_if(&mut self, condition: Match, goto: Match) -> ParseRes {
        dbg!("enter if");
        dbg!(condition.as_str());
        dbg!(goto.as_str());
        dbg!(IF_COND_RE.as_str());

        self.range = condition.range();

        let cap = IF_COND_RE
            .captures(condition.as_str().trim())
            .unwrap()
            .ok_or("unable to parse if condition")?;

        // parse cs
        [
            // input reg
            ("RDDR", Cs::Base(BaseCs::RDDR)),
            ("RDCR", Cs::Base(BaseCs::RDCR)),
            ("RDIP", Cs::Base(BaseCs::RDIP)),
            ("RDSP", Cs::Base(BaseCs::RDSP)),
            ("RDAC", Cs::Base(BaseCs::RDAC)),
            ("RDBR", Cs::Base(BaseCs::RDBR)),
            ("RDPS", Cs::Base(BaseCs::RDPS)),
            ("RDIR", Cs::Base(BaseCs::RDIR)),
            // comp
            ("ZERO", Cs::Comp(false)),
            ("ONE", Cs::Comp(true)),
        ]
        .into_iter()
        .for_each(|(name, cs)| {
            cap.name(name).map(|_| self.add_state(cs));
        });

        self.parse_bit(cap.name("IF_BIT").unwrap())?;
        self.parse_goto(goto)
    }

    fn parse_bit(&mut self, bit: Match) -> ParseRes {
        dbg!("enter bit");
        self.range = bit.range();
        let mut bits: u8 = 0;
        let mut comm = Option::<BaseCs>::default();
        for s in bit.as_str().split(',') {
            let mut bit = match s.trim() {
                "C" => 0,
                "V" => 1,
                "Z" => 2,
                "N" => 3,
                "EI" => 5,
                "INT" => 6,
                "IRQ" => 6,
                "W" => 7,
                "P" => 8,
                s => parse(s).map_err(|_| "unable to parse bit value")?,
            };
            if bit >= 16 {
                Err(format!("can only use bits in range [0, 15], but found `{bit}`").as_str())?
            }
            match (comm, if bit < 8 { BaseCs::LTOL } else { BaseCs::HTOL }) {
                (None, val) => comm = Some(val),
                (Some(old), val) if old != val => {
                    Err("cannot activate lower and upper bytes at the same time")?
                }
                _ => {}
            }

            if bit >= 8 {
                bit -= 8;
            }
            bits |= 1 << bit;
        }
        self.add_state(Cs::Base(comm.ok_or("no bits was selected")?));
        self.add_state(Cs::Bit(bits));
        Ok(())
    }

    fn parse_goto(&mut self, goto: Match) -> ParseRes {
        dbg!("enter goto");

        self.range = goto.range();
        let t = GOTO_RE
            .captures(goto.as_str().trim())
            .unwrap()
            .ok_or("invalid goto body")?;

        let addr = t.get(2).unwrap().as_str();
        self.add_state(Cs::Addr(u8::from_str_radix(addr, 16).map_err(|_| {
            format!("unable to parse address, must be in [00, FF], but got `{addr}`")
        })?));

        Ok(())
    }

    fn sub<'a>(&'a mut self, mtch: Match<'a>) -> Result<(bool, &str), ParseErr> {
        let s = mtch.as_str().trim();
        if let Ok(Some(cap)) = REG_FULL_RE.captures(s) {
            let alu_do = match (cap.name("REG_L"), cap.name("REG_H")) {
                (Some(_), None) => {
                    self.add_state(Cs::Base(BaseCs::LTOL));
                    false
                }
                (None, Some(_)) => {
                    self.add_state(Cs::Base(BaseCs::HTOL));
                    false
                }
                (None, None) => true,
                _ => unreachable!(),
            };
            let s = cap.name("REG").unwrap().as_str();
            return Ok((alu_do, s));
        } else {
            Err(format!("unable to parse register `{s}`"))?
        }
    }

    fn parse_omc(&mut self, s: &str) -> ParseRes {
        for s in s.split(';').filter(|s| s.len() != 0) {
            if let Ok(Some(cap)) = SINGLE_RE.captures(s.trim()) {
                [
                    ("LOAD", Cs::Base(BaseCs::LOAD)),
                    ("STOR", Cs::Base(BaseCs::STOR)),
                    ("HALT", Cs::Base(BaseCs::HALT)),
                    ("IO", Cs::Base(BaseCs::IO)),
                    ("INTS", Cs::Base(BaseCs::INTS)),
                ]
                .into_iter()
                .for_each(|(name, cs)| {
                    cap.name(name).map(|_| self.add_state(cs));
                });
            } else if let Ok(Some(cap)) = OMC_RE.captures(s.trim()) {
                let mut read = cap.get(2).unwrap();

                // parse commutator
                let comm_do = if let Ok(Some(cap)) = COMMUTATOR_RE.captures(read.as_str().trim()) {
                    read = cap.name("OP").or(cap.name("OP1")).unwrap();
                    let f = cap.name("F").or(cap.name("F1")).unwrap().as_str();
                    let v: Vec<BaseCs> = match f {
                        "LTOL" => vec![BaseCs::LTOL],
                        "LTOH" => vec![BaseCs::LTOH],
                        "HTOL" => vec![BaseCs::HTOL],
                        "HTOH" => vec![BaseCs::HTOH],
                        "SWAB" => vec![BaseCs::LTOH, BaseCs::HTOL],
                        "EXTEND SIGN" => vec![BaseCs::SEXT],
                        "SEXT" => vec![BaseCs::SEXT],
                        "ROR" => vec![BaseCs::SHRT, BaseCs::SHRF],
                        "ROL" => vec![BaseCs::SHLT, BaseCs::SHL0],
                        "ASR" => vec![BaseCs::SHRT],
                        "ASL" => vec![BaseCs::SHLT],
                        "SHR" => vec![BaseCs::SHRT],
                        "SHL" => vec![BaseCs::SHLT],
                        s => Err(format!("unable to parse commutator function `{s}`"))?,
                    };
                    v.iter().for_each(|&cs| self.add_state(Cs::Base(cs)));
                    true
                } else {
                    false
                };

                // parse alu
                let alu_do = if let Some(cap) = ALU_WRAPPED_RE
                    .captures(read.as_str().trim())
                    .unwrap()
                    .or(ALU_RE.captures(read.as_str().trim()).unwrap())
                {
                    [
                        ("COML", Cs::Base(BaseCs::COML)),
                        ("COMR", Cs::Base(BaseCs::COMR)),
                        ("SORA", Cs::Base(BaseCs::SORA)),
                        ("PLS1", Cs::Base(BaseCs::PLS1)),
                    ]
                    .into_iter()
                    .for_each(|(name, cs)| {
                        cap.name(name).map(|_| self.add_state(cs));
                    });

                    let left = cap.name("ALU_L");
                    let right = cap.name("ALU_R");

                    match (left, right) {
                        (Some(left), Some(right)) => {
                            let (mut alu_do_right, right) = self.sub(right)?;
                            match right {
                                "DR" => self.add_state(Cs::Base(BaseCs::RDDR)),
                                "CR" => self.add_state(Cs::Base(BaseCs::RDCR)),
                                "IP" => self.add_state(Cs::Base(BaseCs::RDIP)),
                                "SP" => self.add_state(Cs::Base(BaseCs::RDSP)),
                                "0" => alu_do_right = false,
                                s => Err(format!("unable to parse right reg input `{s}`"))?,
                            };
                            let (mut alu_do_left, left) = self.sub(left)?;
                            match left {
                                "AC" => self.add_state(Cs::Base(BaseCs::RDAC)),
                                "BR" => self.add_state(Cs::Base(BaseCs::RDBR)),
                                "PS" => self.add_state(Cs::Base(BaseCs::RDPS)),
                                "IR" => self.add_state(Cs::Base(BaseCs::RDIR)),
                                "0" => alu_do_left = false,
                                s => Err(format!("unable to parse left reg input `{s}`"))?,
                            };
                            alu_do_left || alu_do_right
                        }
                        (Some(arm), None) | (None, Some(arm)) => {
                            let (mut alu_do, arm) = self.sub(arm)?;
                            match arm {
                                "DR" => self.add_state(Cs::Base(BaseCs::RDDR)),
                                "CR" => self.add_state(Cs::Base(BaseCs::RDCR)),
                                "IP" => self.add_state(Cs::Base(BaseCs::RDIP)),
                                "SP" => self.add_state(Cs::Base(BaseCs::RDSP)),
                                "AC" => self.add_state(Cs::Base(BaseCs::RDAC)),
                                "BR" => self.add_state(Cs::Base(BaseCs::RDBR)),
                                "PS" => self.add_state(Cs::Base(BaseCs::RDPS)),
                                "IR" => self.add_state(Cs::Base(BaseCs::RDIR)),
                                "0" => alu_do = false,
                                s => Err(format!("unable to parse reg input `{s}`"))?,
                            };
                            alu_do
                        }
                        (None, None) => unreachable!(),
                    }
                } else {
                    Err(format!("unable to parse ALU actions"))?
                };

                if alu_do && !comm_do {
                    self.add_state(Cs::Base(BaseCs::LTOL));
                    self.add_state(Cs::Base(BaseCs::HTOH));
                }

                // parse write
                for s in cap.get(3).unwrap().as_str().trim().split(',') {
                    let write = Cs::Base(match s.trim() {
                        "DR" => BaseCs::WRDR,
                        "CR" => BaseCs::WRCR,
                        "IP" => BaseCs::WRIP,
                        "SP" => BaseCs::WRSP,
                        "AC" => BaseCs::WRAC,
                        "BR" => BaseCs::WRBR,
                        "PS" => BaseCs::WRPS,
                        "AR" => BaseCs::WRAR,
                        "C" => BaseCs::SETC,
                        "V" => BaseCs::SETV,
                        "N" => BaseCs::STNZ, // goes in pair
                        "Z" => BaseCs::STNZ, // goes in pair
                        s => Err(format!("unable to parse omc write value `{s}`"))?,
                    });
                    self.add_state(write);
                }
            } else {
                Err("unable to parse omc")?
            }
        }

        Ok(())
    }
}

type Css = Vec<Cs>;
impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = ParseState::default();
        dbg!(s);
        state
            .parse_general(s.to_uppercase().as_str())
            .map_err(|e| format!("{e} at `{s}`"))?;
        Ok(Command::new(state.cmd))
    }
}

fn get_alu(base: EnumSet<BaseCs>) -> String {
    let nullify = |s: String| if s.len() == 0 { "0".into() } else { s };
    let mut right = base
        .iter()
        .map(|item| match item {
            BaseCs::RDDR => "DR",
            BaseCs::RDCR => "CR",
            BaseCs::RDIP => "IP",
            BaseCs::RDSP => "SP",
            _ => "",
        })
        .filter(|s| s.len() != 0)
        .join(" | ");
    if base.contains(BaseCs::COMR) {
        right = format!("~{}", nullify(right));
    }

    let mut left = base
        .iter()
        .map(|item| match item {
            BaseCs::RDAC => "AC",
            BaseCs::RDBR => "BR",
            BaseCs::RDPS => "PS",
            BaseCs::RDIR => "IR",
            _ => "",
        })
        .filter(|s| s.len() != 0)
        .join(" | ");
    if base.contains(BaseCs::COML) {
        left = format!("~{}", nullify(left));
    }

    let alu = if base.contains(BaseCs::SORA) {
        format!("{} & {}", nullify(left), nullify(right))
    } else {
        let pls1 = base.contains(BaseCs::PLS1);
        [left, right, (if pls1 { "1" } else { "" }).into()]
            .iter()
            .filter(|s| s.len() != 0)
            .join(" + ")
    };
    alu
}

fn get_alu_full(base: EnumSet<BaseCs>) -> String {
    let alu = get_alu(base);

    let read = match base {
        base if enum_set!(BaseCs::HTOH | BaseCs::LTOL).is_subset(base) => format!("{alu}"),
        base if enum_set!(BaseCs::HTOH).is_subset(base) => format!("HTOH({alu})"),
        base if enum_set!(BaseCs::LTOL | BaseCs::SEXT).is_subset(base) => {
            format!("extend sign {alu}(0..7)")
        }
        base if enum_set!(BaseCs::LTOL).is_subset(base) => format!("LTOL({alu})"),
        base if enum_set!(BaseCs::LTOH | BaseCs::HTOL).is_subset(base) => {
            format!("SWAB({alu})")
        }
        base if enum_set!(BaseCs::LTOH).is_subset(base) => format!("LTOH({alu})"),
        base if enum_set!(BaseCs::HTOL).is_subset(base) => format!("HTOL({alu})"),
        base if enum_set!(BaseCs::SEXT).is_subset(base) => format!("SEXT({alu})"),
        base if enum_set!(BaseCs::SHLT | BaseCs::SHL0).is_subset(base) => {
            format!("ROL({alu})")
        }
        base if enum_set!(BaseCs::SHLT).is_subset(base) => format!("SHL({alu})"),
        base if enum_set!(BaseCs::SHLT | BaseCs::SHRF).is_subset(base) => {
            format!("ROL({alu})")
        }
        base if enum_set!(BaseCs::SHLT).is_subset(base) => format!("SHL({alu})"),
        base if enum_set!(BaseCs::SHRT | BaseCs::SHRF).is_subset(base) => {
            format!("ROR({alu})")
        }
        base if enum_set!(BaseCs::SHRT).is_subset(base) => format!("ASR({alu})"),
        _ => format!("0"),
    };

    read
}

impl ToString for Omc {
    fn to_string(&self) -> String {
        let mut v = vec![];

        let mut write = self
            .base
            .iter()
            .map(|item| match item {
                BaseCs::WRDR => "DR",
                BaseCs::WRCR => "CR",
                BaseCs::WRIP => "IP",
                BaseCs::WRSP => "SP",
                BaseCs::WRAC => "AC",
                BaseCs::WRBR => "BR",
                BaseCs::WRPS => "PS",
                BaseCs::WRAR => "AR",
                _ => "",
            })
            .chain(self.base.contains(BaseCs::STNZ).then(|| "N, Z").into_iter())
            .chain(self.base.contains(BaseCs::SETV).then(|| "V").into_iter())
            .chain(self.base.contains(BaseCs::SETC).then(|| "C").into_iter())
            .filter(|s| s.len() != 0);

        let write = write.join(", ");
        if !write.is_empty() {
            let read = get_alu_full(self.base);
            v.push(format!("{read} → {write}"))
        }
        self.base
            .iter()
            .filter_map(|item| match item {
                BaseCs::LOAD => Some("MEM(AR) → DR"),
                BaseCs::STOR => Some("DR → MEM(AR)"),
                BaseCs::INTS => Some("IRQSC"),
                BaseCs::HALT => Some("Halt"),
                BaseCs::IO => Some("IO"),
                _ => None,
            })
            .for_each(|item| v.push(item.into()));

        v.join("; ")
    }
}

impl Cmc {
    fn goto_to_string(&self, table: Option<&Table>) -> String {
        let addr = self.addr.0;
        let label = table
            .and_then(|table| {
                table
                    .get(addr as usize)
                    .map(|item| item.label)
                    .and_then(|label| (label.len() != 0).then(|| format!("{label} @ ")))
            })
            .unwrap_or("".into());
        format!("GOTO {label}{addr:02X}")
    }

    fn to_string(&self, table: Option<&Table>) -> String {
        // const UNCOND_JUMP: EnumSet<BaseCs> = ;
        if self.bit.0 == 0b10000
            && self.comp.0 == false
            && self.base == enum_set!(BaseCs::RDPS | BaseCs::LTOL)
        {
            return self.goto_to_string(table);
        }

        let alu = get_alu(self.base);
        let offset = if self.base.contains(BaseCs::HTOL) {
            8
        } else {
            0
        };
        let bits = (0..8)
            .filter_map(|bit| ((self.bit.0 & (1 << bit)) != 0).then(|| bit + offset))
            .map(|bit| {
                return if self.base.contains(BaseCs::RDPS) {
                    (match bit {
                        0 => "C",
                        1 => "V",
                        2 => "Z",
                        3 => "N",
                        4 => "4",
                        5 => "EI",
                        6 => "IRQ",
                        7 => "W",
                        8 => "P",
                        _ => unreachable!(),
                    })
                    .into()
                } else {
                    format!("{bit}")
                };
            })
            .join(", ");
        // let bits = ;
        format!(
            "if {alu}({}) = {} then {}",
            bits,
            self.comp.0 as u8,
            self.goto_to_string(table)
        )
    }
}
impl ToString for Cmc {
    fn to_string(&self) -> String {
        self.to_string(None)
    }
}
impl Command {
    pub fn to_string(&self, table: Option<&Table>) -> String {
        let mc: Mc = Into::<Mc>::into(*self);
        return match mc {
            Mc::Operational(omc) => omc.to_string(),
            Mc::Control(cmc) => cmc.to_string(table),
        };
    }
}
