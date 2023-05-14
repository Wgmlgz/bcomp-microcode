use const_format::formatcp;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use paste::paste;

macro_rules! sus {
    ($name:ident, $exp:expr) => {
        pub const $name: &str = formatcp!("(?<{}>{})", stringify!($name), formatcp!($exp));
        paste! {
            lazy_static! {
                pub static ref [< $name _RE>]: Regex = Regex::new(formatcp!("^{}$", $name)).unwrap();
                pub static ref [< $name _RE_UNBOUNDED>]: Regex = Regex::new($name).unwrap();
            }
        }
    };
}

// Primitives
// reg read
// alu right
sus!(RDDR, "DR");
sus!(RDCR, "CR");
sus!(RDIP, "IP");
sus!(RDSP, "SP");
sus!(ALU_R, "{RDDR}|{RDCR}|{RDIP}|{RDSP}|0");
// alu left
sus!(RDAC, "AC");
sus!(RDBR, "BR");
sus!(RDPS, "PS");
sus!(RDIR, "IR");
sus!(ALU_L, "{RDAC}|{RDBR}|{RDPS}|{RDIR}|0");

// ------- CMC -------
// command type 1 pass
sus!(IF_TYPE, "IF(.+)");
sus!(GOTO_TYPE, "GOTO(.+)");

// command type 2 pass
sus!(IF_STRUCTURE, "(.+)THEN(.+)");
sus!(GOTO, r#"GOTO.+@ +([0-9a-fA-F]{{1,2}})"#);

sus!(ONE, "1");
sus!(ZERO, "0");
sus!(COMP, "{ONE}|{ZERO}");
sus!(
    IF_COND,
    r#"({RDDR}|{RDCR}|{RDIP}|{RDSP}|{RDAC}|{RDBR}|{RDPS}|{RDIR})\((?<IF_BIT>.+)\) *= *{COMP}"#
);

sus!(C, "C");
sus!(V, "V");
sus!(Z, "Z");
sus!(N, "N");
sus!(EI, "EI");
sus!(INT, "INT");
sus!(W, "W");
sus!(P, "P");
sus!(N_16, r#"[0-9]{{1,2}}"#);

sus!(IF_BIT, r#"{C}|{V}|{Z}|{N}|{EI}|{INT}|{W}|{P}|{N_16}"#);

// ------- OMC -------
// single cs
sus!(LOAD, r#"MEM\(AR\) +(->|→) +DR"#);
sus!(STOR, r#"DR +(->|→) +MEM\(AR\)"#);
sus!(HALT, r#"HALT"#);
sus!(IO, r#"IO"#);
sus!(INTS, "INTS|IRQSC");
sus!(SINGLE, "{LOAD}|{STOR}|{HALT}|{IO}|{INTS}");

sus!(OMC, r#"(.+)(?:->|→)(.+)"#);

// commutator
sus!(LTOL, "LTOL");
sus!(LTOH, "LTOH");
sus!(HTOL, "HTOL");
sus!(HTOH, "HTOH");
sus!(SWAB, "SWAB");
sus!(SEXT, "EXTEND SIGN");
sus!(ROR, "ROR");
sus!(ROL, "ROL");
sus!(ASR, "ASR");
sus!(ASL, "ASL");
sus!(OPS, "{LTOL}|{LTOH}|{HTOL}|{HTOH}|{SWAB}|{SEXT}|{ROR}|{ROL}|{ASR}|{ASL}");
sus!(COMMUTATOR, r#"((?<F1>{OPS})(?<OP1>.*))|((?<F>.+)\((?<OP>.*)\))"#);


// alu ops
sus!(ALU, r#"(?<COML>~)?(?<ALU_L>[^+&~1]+) *((\+|(?<SORA>&)) *(?<COMR>~)?(?<ALU_R>[^+&~1]+))? *(?<PLS1>\+ *1)?"#);
sus!(ALU_WRAPPED, r#"\({ALU}\)"#);


sus!(REG_L, r#"\(0\.\.7\)"#);
sus!(REG_H, r#"\(8\.\.15\)"#);
sus!(REG, r#"[A-Z0]+"#);
sus!(REG_FULL, r#"({REG}({REG_L}|{REG_H})?)|{ZERO}"#);

// sus!(ALU_R_FULL, r#"(?<COMR>~)?{ALU_R}"#);
// sus!(ALU_L_FULL, r#"(?<COML>~)?{ALU_L}"#);
// sus!(PLS1, r#"\k<ALU_L_FULL> \+ \k<ALU_R_FULL> + 1"#);
// sus!(SORA, r#"\k<ALU_L_FULL> & \k<ALU_R_FULL>"#);
// sus!(ADD, r#"\k<ALU_L_FULL> + \k<ALU_R_FULL>"#); // implicit activation if PLS1 and SORA are deactivated
// sus!(ALU_RES, "{ALU_R_FULL}|{ALU_L_FULL}|{PLS1}|{SORA}|{ADD}"); // helper

// // todo commutator
// // reg write
// sus!(WRDR, "DR");
// sus!(WRCR, "CR");
// sus!(WRIP, "IP");
// sus!(WRSP, "SP");
// sus!(WRAC, "AC");
// sus!(WRBR, "BR");
// sus!(WRPS, "PS");
// sus!(WRAR, "AR");
// sus!(
//     REG_WR,
//     "{WRDR}|{WRCR}|{WRIP}|{WRSP}|{WRAC}|{WRBR}|{WRPS}|{WRAR}"
// );
