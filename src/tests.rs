#[cfg(test)]

mod tests {
    use duplicate::duplicate_item;

    use crate::{table::TABLE, Command};
    use paste::paste;
    use seq_macro::seq;

    seq! {address in 0x00..=0xff {
        paste! {
            #[test]
            fn [<decode_ address>]() {
                let instr = &TABLE[address];
                let command: Command = (instr.decoded).parse().unwrap();
                dbg!(Command::new(instr.encoded).cs());
                dbg!(command.cs());
                dbg!(&instr);
                assert_eq!(format!("{:x}", instr.encoded), format!("{:x}", command.cmd))
            }

            #[test]
            fn [<encode_ address>]() {
                let instr = &TABLE[address];
                let command: Command = Command::new(instr.encoded);
                dbg!(Command::new(instr.encoded).cs());
                dbg!(command.cs());
                dbg!(&instr);
                assert_eq!(format!("{}", instr.decoded), format!("{}", command.to_string(Some(&TABLE))))
            }
        }
    }}
}
