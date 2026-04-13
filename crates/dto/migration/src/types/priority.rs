use sea_orm::DeriveValueType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType, Default, PartialOrd, Ord)]
#[sea_orm(value_type = "String")]
pub enum Priority {
    Asap,
    High,
    #[default]
    Medium,
    Low,
    Far,
}

#[derive(Debug)]
pub struct PriorityParseError {
    bad: String,
}

impl std::error::Error for PriorityParseError {}

impl std::fmt::Display for PriorityParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not a valid Priority!", self.bad)
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Priority::Asap => "ASAP",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
            Priority::Far => "Far",
        };

        write!(f, "{str}")
    }
}

impl std::str::FromStr for Priority {
    type Err = PriorityParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            "ASAP" => Self::Asap,
            "High" => Self::High,
            "Medium" => Priority::Medium,
            "Low" => Self::Low,
            "Far" => Self::Far,
            _ => return Err(PriorityParseError { bad: s.to_owned() }),
        };

        Ok(v)
    }
}
