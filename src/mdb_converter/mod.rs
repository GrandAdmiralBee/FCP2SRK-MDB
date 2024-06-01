pub mod mdb_parser;
pub mod parser;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum FCP {
    SE,
    ASM,
    DP,
    DRC,
    EDIF,
    ERP,
    GDSREADER,
    IO,
    LMAN,
    ME,
    REPORTS,
    SDB,
    SHELL,
    TDM,
    UI,
}

impl FCP {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fcpasm" => Some(Self::ASM),
            "fcpdp" => Some(Self::DP),
            "fcpdrc" => Some(Self::DRC),
            "fcpedif" => Some(Self::EDIF),
            "fcperp" => Some(Self::ERP),
            "fcpgdsreader" => Some(Self::GDSREADER),
            "fcpio" => Some(Self::IO),
            "fcplman" => Some(Self::LMAN),
            "fcpme" => Some(Self::ME),
            "fcpreports" => Some(Self::REPORTS),
            "fcpsdb" => Some(Self::SDB),
            "fcpse" => Some(Self::SE),
            "fcpshell" => Some(Self::SHELL),
            "fcptdm" => Some(Self::TDM),
            "fcpui" => Some(Self::UI),
            _ => None,
        }
    }

    pub fn to_str(&self) -> String {
        let s = match self {
            Self::ASM => "fcpasm",
            Self::DP => "fcpdp",
            Self::DRC => "fcpdrc",
            Self::EDIF => "fcpedif",
            Self::ERP => "fcperp",
            Self::GDSREADER => "fcpgdsreader",
            Self::IO => "fcpio",
            Self::LMAN => "fcplman",
            Self::ME => "fcpme",
            Self::REPORTS => "fcpreports",
            Self::SDB => "fcpsdb",
            Self::SE => "fcpse",
            Self::SHELL => "fcpshell",
            Self::TDM => "fcptdm",
            Self::UI => "fcpui",
        };
        s.to_string()
    }
}
