use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::num;

use truthtable::*;
use var::*;

const SYN: u32 = 0x890;
const AC0: u32 = 0x891;

#[derive(Clone, Debug)]
struct Output {
    ac1: u32,       // Single fuse
    xor: u32,       // Single fuse
    ptd: u32,       // Array of fuses, same length as pts
    pts: Vec<u32>,  // Array of array of fuses, each subarray is the length of INPUTS
}

// Output modes supported:
//  SYN=1 AC0=0 AC1=1   disabled (always high impedence)
//  SYN=1 AC0=0 AC1=0   combinatorial (always low impedence)

#[derive(Clone, Debug)]
enum OutputMode {
    Disabled,
    Combinatorial {
        tt: TruthTable,
    },
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputMode::Disabled => write!(f, "disabled"),
            OutputMode::Combinatorial {tt} => write!(f, "always-enabled combinatorial\n{}", tt),
        }
    }
}

lazy_static! {
    static ref INPUTS: BTreeMap<u32, u32> = {
        let mut m = BTreeMap::new();
        m.insert(2, 0);
        m.insert(19, 1);
        m.insert(3, 2);
        m.insert(18, 3);
        m.insert(4, 4);
        m.insert(17, 5);
        m.insert(5, 6);
        m.insert(16, 7);
        m.insert(6, 8);
        m.insert(15, 9);
        m.insert(7, 10);
        m.insert(14, 11);
        m.insert(8, 12);
        m.insert(13, 13);
        m.insert(9, 14);
        m.insert(12, 15);
        m
    };

    static ref OUTPUTS: BTreeMap<u32, Output> = {
        let mut m = BTreeMap::new();
        m.insert(19, Output { ac1: 0x848, xor: 0x800, ptd: 0x850, pts: vec![0x000, 0x020, 0x040, 0x060, 0x080, 0x0a0, 0x0c0, 0x0e0] });
        m.insert(18, Output { ac1: 0x849, xor: 0x801, ptd: 0x858, pts: vec![0x100, 0x120, 0x140, 0x160, 0x180, 0x1a0, 0x1c0, 0x1e0] });
        m.insert(17, Output { ac1: 0x84a, xor: 0x802, ptd: 0x860, pts: vec![0x200, 0x220, 0x240, 0x260, 0x280, 0x2a0, 0x2c0, 0x2e0] });
        m.insert(16, Output { ac1: 0x84b, xor: 0x803, ptd: 0x868, pts: vec![0x300, 0x320, 0x340, 0x360, 0x380, 0x3a0, 0x3c0, 0x3e0] });
        m.insert(15, Output { ac1: 0x84c, xor: 0x804, ptd: 0x870, pts: vec![0x400, 0x420, 0x440, 0x460, 0x480, 0x4a0, 0x4c0, 0x4e0] });
        m.insert(14, Output { ac1: 0x84d, xor: 0x805, ptd: 0x878, pts: vec![0x500, 0x520, 0x540, 0x560, 0x580, 0x5a0, 0x5c0, 0x5e0] });
        m.insert(13, Output { ac1: 0x84e, xor: 0x806, ptd: 0x880, pts: vec![0x600, 0x620, 0x640, 0x660, 0x680, 0x6a0, 0x6c0, 0x6e0] });
        m.insert(12, Output { ac1: 0x84f, xor: 0x807, ptd: 0x888, pts: vec![0x700, 0x720, 0x740, 0x760, 0x780, 0x7a0, 0x7c0, 0x7e0] });
        m
    };
}

pub struct GAL16V8 {
    inputs: BTreeMap<Var, u32>,
    outputs: BTreeMap<u32, OutputMode>,
}

impl GAL16V8 {
    pub fn new() -> GAL16V8 {
        GAL16V8 {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }

    pub fn input(&mut self, pin: u32, name: &str) -> TruthTable {
        let v = Var::from(name);
        if let Some(inum) = INPUTS.get(&pin) {
            if let Some(opin) = self.inputs.get(&v) {
                if pin != *opin {
                    panic!("Variable {} is already used for pin {}, cannot assign to pin {}", v, opin, pin);
                }
            } else {
                self.inputs.insert(v.clone(), *inum);
            }

            TruthTable::from(name)
        } else {
            panic!("Cannot configure illegal input pin {}", pin);
        }
    }

    pub fn disable_output(&mut self, pin: u32) {
        if !OUTPUTS.contains_key(&pin) {
            panic!("Cannot configure illegal output pin {}", pin);
        }

        if let Some(mode) = self.outputs.get(&pin) {
            match *mode {
                OutputMode::Disabled => (),
                _ => panic!("Cannot configure output pin {} already set with mode {}", pin, mode),
            }
        } else {
            self.outputs.insert(pin, OutputMode::Disabled);
        }
    }

    pub fn combinatorial_output(&mut self, pin: u32, tt: TruthTable) {
        if !OUTPUTS.contains_key(&pin) {
            panic!("Cannot configure illegal output pin {}", pin);
        }

        if let Some(mode) = self.outputs.get(&pin) {
            match mode {
                OutputMode::Combinatorial{tt: ott, ..} => {
                    if tt != *ott {
                        panic!("Cannot configure output pin {} already set to a different truth table!\nCurrent table:\n{}\nWant to set:\n{}", pin, ott, tt);
                    }
                },
                _ => panic!("Cannot configure output pin {} already set with mode {}", pin, mode),
            }
        } else {
            self.outputs.insert(pin, OutputMode::Combinatorial{tt: tt});
        }
    }

    #[must_use]
    pub fn write(&self, f: &mut dyn io::Write) -> io::Result<()> {
        let mut checksum = num::Wrapping(0);
        fn write(checksum: &mut num::Wrapping<u16>, f: &mut dyn io::Write, s: String) -> io::Result<()> {
            for c in s.chars() {
                *checksum += num::Wrapping(c as u16);
            }
            write!(f, "{}", s)
        }

        macro_rules! out {
            ($($x:tt)*) => (write(&mut checksum, f, format!($($x)*)));
        }

        out!("\x02\n\n*N GAL16V8 fuse layout\n  *F0 *G0 *QF2194\n\n")?;

        out!("*N Simple mode\n  *L{:0>4} 1 *L{:0>4} 0\n", SYN, AC0)?;

        for (pin, fuses) in OUTPUTS.iter() {
            out!("\n*N Macrocell for pin {}\n", pin)?;

            if let Some(mode) = self.outputs.get(pin) {
                match mode {

                    OutputMode::Disabled => out!("  *N Unused *L{:0>4} 1\n", fuses.ac1)?,

                    OutputMode::Combinatorial{tt} => {
                        out!("  *N Combinatorial *L{:0>4} 0\n", fuses.ac1)?;

                        let prod;
                        let pos_prod = tt.dnf();
                        let mut neg_prod = (!tt).dnf();
                        neg_prod.invert = !neg_prod.invert;
                        if neg_prod.terms.len() < pos_prod.terms.len() {
                            prod = neg_prod;
                        } else {
                            prod = pos_prod;
                        }

                        if prod.terms.len() >= fuses.pts.len() {
                            panic!("Too many terms in product for this macrocell! (needs {}, has {})", prod.terms.len(), fuses.pts.len());
                        }

                        if prod.invert {
                            out!("  *N Negative polarity *L{:0>4} 0\n", fuses.xor)?;
                        } else {
                            out!("  *N Positive polarity *L{:0>4} 1\n", fuses.xor)?;
                        }

                        for (i, term) in prod.terms.iter().enumerate() {
                            out!("  *L{:0>4} 1 *L{:0>4} ", fuses.ptd + (i as u32), fuses.pts[i])?;

                            let mut ordered_term: Vec<Factor> = (0..INPUTS.len()).map(|_| Factor::DontCare).collect();

                            for (i, factor) in term.iter().enumerate() {
                                match factor {
                                    Factor::DontCare => {},
                                    Factor::IsFalse => {
                                        if let Some(inum) = self.inputs.get(tt.var(i)) {
                                            ordered_term[*inum as usize] = Factor::IsFalse;
                                        } else {
                                            panic!("Output pin {} depends on variable {} which is not an input", pin, tt.var(i));
                                        }
                                    },
                                    Factor::IsTrue => {
                                        if let Some(inum) = self.inputs.get(tt.var(i)) {
                                            ordered_term[*inum as usize] = Factor::IsTrue;
                                        } else {
                                            panic!("Output pin {} depends on variable {} which is not an input", pin, tt.var(i));
                                        }
                                    },
                                }
                            }

                            for factor in ordered_term {
                                match factor {
                                    Factor::DontCare => out!("11")?,
                                    Factor::IsFalse => out!("10")?,
                                    Factor::IsTrue => out!("01")?,
                                }
                            }

                            out!("\n")?;
                        }
                    },

                }
            } else {
                out!("  *N Unused *L{:0>4} 0\n", fuses.ac1)?;
            }
        }

        out!("\n*N End of image.\n\n\x03")?;

        write!(f, "{:0>4X}\n", checksum)?;

        Ok(())
    }
}
