#[derive(Debug, Clone, PartialEq)]
pub struct SceneDesc {
    pub statements: Vec<Statement>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

impl ToString for Statement {
    fn to_string(&self) -> String {
        let body = self.body
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        format!("{}({}){{{}}}", self.name, self.args.join(", "), body.join("; "))
    }
}
