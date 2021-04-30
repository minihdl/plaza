use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt;
use std::ops;

use var::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Factor {
    DontCare,
    IsFalse,
    IsTrue,
}

#[derive(Clone, Debug)]
pub struct Product {
    pub invert: bool,
    pub terms: Vec<Vec<Factor>>,
}

#[derive(Clone, Debug)]
pub enum TruthTable {
    AlwaysTrue,
    AlwaysFalse,
    Explicit {
        vars: Vec<Var>,
        table: Vec<bool>,
    },
}

impl TruthTable {
    pub fn always() -> TruthTable { TruthTable::AlwaysTrue }
    pub fn never() -> TruthTable { TruthTable::AlwaysFalse }

    fn true_rows(&self) -> usize {
        match self {
            TruthTable::AlwaysTrue => 0,
            TruthTable::AlwaysFalse => 0,
            TruthTable::Explicit{table, ..} => {
                let mut n = 0;
                for r in table {
                    if *r { n += 1 }
                }
                n
            },
        }
    }

    pub fn var(&self, i: usize) -> &Var {
        match self {
            TruthTable::AlwaysTrue => panic!("Truth table does not have variable {}", i),
            TruthTable::AlwaysFalse => panic!("Truth table does not have variable {}", i),
            TruthTable::Explicit{vars, ..} => &vars[i],
        }
    }

    pub fn dnf(&self) -> Product {
        match self {
            TruthTable::AlwaysTrue => Product { invert: true, terms: Vec::new() },

            TruthTable::AlwaysFalse => Product { invert: false, terms: Vec::new() },

            TruthTable::Explicit{vars, ..} => {
                let mut terms = Vec::new();
                let mut product = TruthTable::never();

                while self != &product {
                    let mut bestterm = vars.iter().map(|_| Factor::DontCare).collect();
                    let mut besttable = TruthTable::never();
                    let mut bestscore = 0;

                    let term_iter = vars.iter().map(|_| vec![Factor::DontCare, Factor::IsFalse, Factor::IsTrue]);
                    for term in term_iter.multi_cartesian_product() {
                        let mut term_table = TruthTable::always();

                        for (i, f) in term.iter().enumerate() {
                            match f {
                                Factor::DontCare => {},
                                Factor::IsFalse => { term_table &= !TruthTable::from(&*vars[i].name) },
                                Factor::IsTrue => { term_table &= TruthTable::from(&*vars[i].name) },
                            }
                        }

                        if (&term_table & !self) != TruthTable::never() {
                            continue;
                        }

                        let score = (&product | &term_table).true_rows();

                        if score > bestscore {
                            bestterm = term;
                            besttable = term_table;
                            bestscore = score;
                        }
                    }

                    terms.push(bestterm);
                    product |= besttable;
                }

                Product { invert: false, terms: terms }
            },
        }
    }
}

impl From<&str> for TruthTable {
    fn from(name: &str) -> TruthTable {
        let v = Var::from(name);
        TruthTable::Explicit {
            vars: vec![v],
            table: vec![false, true],
        }
    }
}

impl fmt::Display for TruthTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TruthTable::AlwaysTrue => write!(f, " * | 1")?,
            TruthTable::AlwaysFalse => write!(f, " * | 0")?,
            TruthTable::Explicit{vars, table} => {
                for v in vars {
                    write!(f, " {}", v)?;
                }
                write!(f, " |\n")?;
                for v in vars {
                    write!(f, "-{:->width$}", "", width=v.len())?;
                }
                write!(f, "-+---")?;
                for (i, r) in table.iter().enumerate() {
                    write!(f, "\n")?;
                    for (j, v) in vars.iter().enumerate() {
                        write!(f, " {: >width$}", if i & (1 << j) != 0 { '1' } else { '0' }, width=v.len())?;
                    }
                    write!(f, " | {}", if *r { '1' } else { '0' })?;
                }
            },
        };
        Ok(())
    }
}

impl PartialEq for TruthTable {
    fn eq(&self, that: &TruthTable) -> bool {
        let mut vs = BTreeMap::new();

        match self {
            TruthTable::Explicit{vars, ..} => {
                for (i, v) in vars.iter().enumerate() {
                    vs.insert(&*v.name, (Some(i), None));
                }
            },
            _ => (),
        };

        match that {
            TruthTable::Explicit{vars, ..} => {
                for (i, v) in vars.iter().enumerate() {
                    vs.entry(&*v.name).or_insert((None, None)).1 = Some(i);
                }
            },
            _ => (),
        };

        if vs.len() == 0 {
            match self {
                TruthTable::AlwaysTrue => match that {
                    TruthTable::AlwaysTrue => return true,
                    TruthTable::AlwaysFalse => return false,
                    TruthTable::Explicit{..} => panic!("Empty explicit truth table"),
                },
                TruthTable::AlwaysFalse => match that {
                    TruthTable::AlwaysTrue => return false,
                    TruthTable::AlwaysFalse => return true,
                    TruthTable::Explicit{..} => panic!("Empty explicit truth table"),
                },
                TruthTable::Explicit{..} => panic!("Empty explicit truth table"),
            }
        }

        for x in 0 .. (1 << vs.len()) {
            let mut lr = 0;
            let mut rr = 0;

            for (i, (_, (li, ri))) in vs.iter().enumerate() {
                let b = x & (1 << i) != 0;
                if b {
                    if let Some(li) = li {
                        lr |= 1 << li;
                    }
                    if let Some(ri) = ri {
                        rr |= 1 << ri;
                    }
                }
            }

            let lv = match self {
                TruthTable::AlwaysTrue => true,
                TruthTable::AlwaysFalse => false,
                TruthTable::Explicit{table, ..} => table[lr],
            };

            let rv = match that {
                TruthTable::AlwaysTrue => true,
                TruthTable::AlwaysFalse => false,
                TruthTable::Explicit{table, ..} => table[rr],
            };

            if lv != rv {
                return false;
            }
        }

        true
    }
}

impl Eq for TruthTable {}

fn bitop(l: &TruthTable, r: &TruthTable, f: &dyn Fn(bool, bool) -> bool) -> TruthTable {
    let mut vs = BTreeMap::new();

    match l {
        TruthTable::Explicit{vars, ..} => {
            for (i, v) in vars.iter().enumerate() {
                vs.insert(&*v.name, (Some(i), None));
            }
        },
        _ => (),
    };

    match r {
        TruthTable::Explicit{vars, ..} => {
            for (i, v) in vars.iter().enumerate() {
                vs.entry(&*v.name).or_insert((None, None)).1 = Some(i);
            }
        },
        _ => (),
    };

    if vs.len() == 0 {
        match l {
            TruthTable::AlwaysTrue => match r {
                TruthTable::AlwaysTrue => if f(true, true) { return TruthTable::AlwaysTrue } else { return TruthTable::AlwaysFalse },
                TruthTable::AlwaysFalse => if f(true, false) { return TruthTable::AlwaysTrue } else { return TruthTable::AlwaysFalse },
                TruthTable::Explicit{..} => panic!("Empty explicit truth table"),
            },
            TruthTable::AlwaysFalse => match r {
                TruthTable::AlwaysTrue => if f(false, true) { return TruthTable::AlwaysTrue } else { return TruthTable::AlwaysFalse },
                TruthTable::AlwaysFalse => if f(false, false) { return TruthTable::AlwaysTrue } else { return TruthTable::AlwaysFalse },
                TruthTable::Explicit{..} => panic!("Empty explicit truth table"),
            },
            TruthTable::Explicit{..} => panic!("Empty explicit truth table"),
        }
    }

    let mut table = Vec::with_capacity(1 << vs.len());

    for x in 0 .. (1 << vs.len()) {
        let mut lr = 0;
        let mut rr = 0;

        for (i, (_, (li, ri))) in vs.iter().enumerate() {
            let b = x & (1 << i) != 0;
            if b {
                if let Some(li) = li {
                    lr |= 1 << li;
                }
                if let Some(ri) = ri {
                    rr |= 1 << ri;
                }
            }
        }

        let lv = match l {
            TruthTable::AlwaysTrue => true,
            TruthTable::AlwaysFalse => false,
            TruthTable::Explicit{table, ..} => table[lr],
        };

        let rv = match r {
            TruthTable::AlwaysTrue => true,
            TruthTable::AlwaysFalse => false,
            TruthTable::Explicit{table, ..} => table[rr],
        };

        table.push(f(lv, rv));
    }

    let vars = vs.iter().map(|x| Var::from(*x.0)).collect();

    TruthTable::Explicit {
        vars: vars,
        table: table,
    }
}

impl ops::BitAnd for &TruthTable {
    type Output = TruthTable;

    fn bitand(self, x: &TruthTable) -> TruthTable { bitop(self, x, &bool::bitand) }
}

impl ops::BitAnd for TruthTable { type Output = TruthTable; fn bitand(self, x: TruthTable) -> TruthTable { &self & &x } }
impl ops::BitAnd<&TruthTable> for TruthTable { type Output = TruthTable; fn bitand(self, x: &TruthTable) -> TruthTable { &self & x } }
impl ops::BitAnd<TruthTable> for &TruthTable { type Output = TruthTable; fn bitand(self, x: TruthTable) -> TruthTable { self & &x } }

impl ops::BitAndAssign for TruthTable { fn bitand_assign(&mut self, x: TruthTable) { *self = &*self & &x } }
impl ops::BitAndAssign<&TruthTable> for TruthTable { fn bitand_assign(&mut self, x: &TruthTable) { *self = &*self & x } }

impl ops::BitOr for &TruthTable {
    type Output = TruthTable;

    fn bitor(self, x: &TruthTable) -> TruthTable { bitop(self, x, &bool::bitor) }
}

impl ops::BitOr for TruthTable { type Output = TruthTable; fn bitor(self, x: TruthTable) -> TruthTable { &self | &x } }
impl ops::BitOr<&TruthTable> for TruthTable { type Output = TruthTable; fn bitor(self, x: &TruthTable) -> TruthTable { &self | x } }
impl ops::BitOr<TruthTable> for &TruthTable { type Output = TruthTable; fn bitor(self, x: TruthTable) -> TruthTable { self | &x } }

impl ops::BitOrAssign for TruthTable { fn bitor_assign(&mut self, x: TruthTable) { *self = &*self | &x } }
impl ops::BitOrAssign<&TruthTable> for TruthTable { fn bitor_assign(&mut self, x: &TruthTable) { *self = &*self | x } }

impl ops::BitXor for &TruthTable {
    type Output = TruthTable;

    fn bitxor(self, x: &TruthTable) -> TruthTable { bitop(self, x, &bool::bitxor) }
}

impl ops::BitXor for TruthTable { type Output = TruthTable; fn bitxor(self, x: TruthTable) -> TruthTable { &self ^ &x } }
impl ops::BitXor<&TruthTable> for TruthTable { type Output = TruthTable; fn bitxor(self, x: &TruthTable) -> TruthTable { &self ^ x } }
impl ops::BitXor<TruthTable> for &TruthTable { type Output = TruthTable; fn bitxor(self, x: TruthTable) -> TruthTable { self ^ &x } }

impl ops::BitXorAssign for TruthTable { fn bitxor_assign(&mut self, x: TruthTable) { *self = &*self ^ &x } }
impl ops::BitXorAssign<&TruthTable> for TruthTable { fn bitxor_assign(&mut self, x: &TruthTable) { *self = &*self ^ x } }

impl ops::Not for &TruthTable {
    type Output = TruthTable;

    fn not(self) -> TruthTable {
        match self {
            TruthTable::AlwaysTrue => TruthTable::AlwaysFalse,
            TruthTable::AlwaysFalse => TruthTable::AlwaysTrue,
            TruthTable::Explicit{vars, table} => TruthTable::Explicit {
                vars: vars.clone(),
                table: table.iter().map(|x| !x).collect(),
            },
        }
    }
}

impl ops::Not for TruthTable {
    type Output = TruthTable;

    fn not(self) -> TruthTable {
        match self {
            TruthTable::AlwaysTrue => TruthTable::AlwaysFalse,
            TruthTable::AlwaysFalse => TruthTable::AlwaysTrue,
            TruthTable::Explicit{vars, mut table} => {
                for x in &mut table {
                    *x = !*x;
                }
                TruthTable::Explicit {
                    vars: vars,
                    table: table,
                }
            },
        }
    }
}
