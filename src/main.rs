mod control_signals;
mod regexes;
mod table;
mod tests;

use control_signals::{
    BaseCs::{self},
    Cs::{self, *},
};
use fancy_regex::Match;
use itertools::Itertools;
use parse_int::parse;
use regexes::*;
use std::{error::Error, ops::Range, str::FromStr};
type ParseErr = Box<dyn Error>;
type ParseRes = Result<(), ParseErr>;

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

use num_traits::{FromPrimitive, Zero};

#[derive(Default)]
pub struct Command {
    pub cmd: u64,
}

impl Command {
    fn new(cmd: u64) -> Self {
        Self { cmd }
    }
    fn cs(&self) -> Vec<Cs> {
        let mc_type = !(self.cmd & (1 << 39)).is_zero();
        let mut v = (if mc_type { 0..16 } else { 0..39 })
            .filter_map(|bit| {
                (!(self.cmd & (1 << bit)).is_zero()).then(|| Base(BaseCs::from_usize(bit).unwrap()))
            })
            .collect_vec();
        if mc_type {
            v.push(Bit(((self.cmd >> 16) & 0xff) as u8));
            v.push(Addr(((self.cmd >> 24) & 0x7f) as u8));
            v.push(Comp(((self.cmd >> 32) & 0x1) != 0));
        }
        v.push(Type(mc_type));

        v
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
            self.add_state(Type(true));

            let cap = IF_STRUCTURE_RE
                .captures(cap.get(2).unwrap().as_str())
                .unwrap()
                .ok_or("seems like if command, but unable to parce it")?;

            return self.parse_if(cap.get(2).unwrap(), cap.get(3).unwrap());
        } else if let Some(cap) = GOTO_TYPE_RE.captures(s)? {
            // that's how unconditional jums works lol
            self.add_state(Type(true));
            self.add_state(Base(BaseCs::RDPS));
            self.add_state(Base(BaseCs::LTOL));
            self.add_state(Bit(0b0010000));
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
            ("RDDR", Base(BaseCs::RDDR)),
            ("RDCR", Base(BaseCs::RDCR)),
            ("RDIP", Base(BaseCs::RDIP)),
            ("RDSP", Base(BaseCs::RDSP)),
            ("RDAC", Base(BaseCs::RDAC)),
            ("RDBR", Base(BaseCs::RDBR)),
            ("RDPS", Base(BaseCs::RDPS)),
            ("RDIR", Base(BaseCs::RDIR)),
            // comp
            ("ZERO", Comp(false)),
            ("ONE", Comp(true)),
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
        self.add_state(Base(comm.ok_or("no bits was selected")?));
        self.add_state(Bit(bits));
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
        self.add_state(Addr(u8::from_str_radix(addr, 16).map_err(|_| {
            format!("unable to parse address, must be in [00, FF], but got `{addr}`")
        })?));

        Ok(())
    }

    fn sub<'a>(&'a mut self, mtch: Match<'a>) -> Result<(bool, &str), ParseErr> {
        let s = mtch.as_str().trim();
        if let Ok(Some(cap)) = REG_FULL_RE.captures(s) {
            let alu_do = match (cap.name("REG_L"), cap.name("REG_H")) {
                (Some(_), None) => {
                    self.add_state(Base(BaseCs::LTOL));
                    false
                }
                (None, Some(_)) => {
                    self.add_state(Base(BaseCs::HTOL));
                    false
                }
                (None, None) => {
                    // self.add_state(Base(BaseCs::HTOH));
                    // self.add_state(Base(BaseCs::LTOL));
                    true
                }
                _ => unreachable!(),
            };
            let s = cap.name("REG").unwrap().as_str();
            return Ok((alu_do, s));
        } else {
            Err(format!("unable to parse register `{s}`"))?
        }
        // let val = val.as_str().trim();
        // let reg: String = val[..val.len().min(2)].into();
    }

    fn parse_omc(&mut self, s: &str) -> ParseRes {
        // self.range = omc.range();
        // self.range.start -= 1;
        for s in s.split(';') {
            // self.range.start += s.len() + 1;
            // self.range.end = self.range.start + s.len();

            if let Ok(Some(cap)) = SINGLE_RE.captures(s.trim()) {
                [
                    ("LOAD", Base(BaseCs::LOAD)),
                    ("STOR", Base(BaseCs::STOR)),
                    ("HALT", Base(BaseCs::HALT)),
                    ("IO", Base(BaseCs::IO)),
                    ("INTS", Base(BaseCs::INTS)),
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
                    v.iter().for_each(|&cs| self.add_state(Base(cs)));
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
                        ("COML", Base(BaseCs::COML)),
                        ("COMR", Base(BaseCs::COMR)),
                        ("SORA", Base(BaseCs::SORA)),
                        ("PLS1", Base(BaseCs::PLS1)),
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
                                "DR" => self.add_state(Base(BaseCs::RDDR)),
                                "CR" => self.add_state(Base(BaseCs::RDCR)),
                                "IP" => self.add_state(Base(BaseCs::RDIP)),
                                "SP" => self.add_state(Base(BaseCs::RDSP)),
                                "0" => alu_do_right = false,
                                s => Err(format!("unable to parse right reg input `{s}`"))?,
                            };
                            let (mut alu_do_left, left) = self.sub(left)?;
                            match left {
                                "AC" => self.add_state(Base(BaseCs::RDAC)),
                                "BR" => self.add_state(Base(BaseCs::RDBR)),
                                "PS" => self.add_state(Base(BaseCs::RDPS)),
                                "IR" => self.add_state(Base(BaseCs::RDIR)),
                                "0" => alu_do_left = false,
                                s => Err(format!("unable to parse left reg input `{s}`"))?,
                            };
                            alu_do_left || alu_do_right
                        }
                        (Some(arm), None) | (None, Some(arm)) => {
                            let (mut alu_do, arm) = self.sub(arm)?;
                            match arm {
                                "DR" => self.add_state(Base(BaseCs::RDDR)),
                                "CR" => self.add_state(Base(BaseCs::RDCR)),
                                "IP" => self.add_state(Base(BaseCs::RDIP)),
                                "SP" => self.add_state(Base(BaseCs::RDSP)),
                                "AC" => self.add_state(Base(BaseCs::RDAC)),
                                "BR" => self.add_state(Base(BaseCs::RDBR)),
                                "PS" => self.add_state(Base(BaseCs::RDPS)),
                                "IR" => self.add_state(Base(BaseCs::RDIR)),
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
                    self.add_state(Base(BaseCs::LTOL));
                    self.add_state(Base(BaseCs::HTOH));
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

impl FromStr for Command {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = ParseState::default();
        dbg!(s);
        state
            .parse_general(s.to_uppercase().as_str())
            .map_err(|e| format!("{e} at `{s}`"))?;
        Ok(Command::new(state.cmd))
    }
}
// impl Command {
//     fn (s: Into<String>) {

//     }
// }

fn main() {
    dbg!(Command::new(0x0020011002).cs());
    // let t =
    // dbg!(OMC);
    // dbg!(MC_RE.as_str());

    // dbg!(OMC);
    // let cap = ALU_RES_RE.captures("DR -> IP").unwrap();
    // let n = ALU_R_FULL_RE.capture_names();
    // dbg!(n);

    // let v = cap.iter().collect_vec();
    // dbg!(v);
    // println!("Hello, world!");
}
