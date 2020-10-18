pub struct Glsl {
    uniq: usize,
    functions: Vec<String>,
}

impl Glsl {
    pub fn new() -> Self {
        Glsl {
            uniq: 0,
            functions: Vec::new(),
        }
    }

    pub fn add_function(
        &mut self,
        typ: impl ToString,
        name: impl ToString,
        args: &[(impl ToString, impl ToString)],
    ) -> Function {
        self.uniq += 1;
        Function {
            uniq_above: self.uniq,
            uniq: 0,
            ret: typ.to_string(),
            name: name.to_string(),
            args: args
                .into_iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            definitions: Vec::new(),
        }
    }
}

impl ToString for Glsl {
    fn to_string(&self) -> String {
        self.functions.join("\n")
    }
}

pub struct Function {
    uniq_above: usize,
    uniq: usize,
    ret: String,
    name: String,
    args: Vec<(String, String)>,
    definitions: Vec<String>,
}

impl Function {
    pub fn gen_definition(&mut self, typ: impl ToString, expr: impl ToString) -> String {
        self.uniq += 1;
        let ident = format!("def_{}_{}", self.uniq_above, self.uniq);

        let def = format!("{} {} = {};", typ.to_string(), ident, expr.to_string());
        self.definitions.push(def);

        ident
    }

    pub fn ret(self, glsl: &mut Glsl, expr: impl ToString) {
        let args = self
            .args
            .into_iter()
            .map(|(typ, name)| format!("{} {}", typ, name))
            .collect::<Vec<_>>();

        let fst = format!("{} {}({}) {{", self.ret, self.name, args.join(", "));

        let definitions = self.definitions.join("\n");

        let ret = format!("return {};", expr.to_string());

        glsl.functions
            .push(format!("{}\n{}\n{}\n}}", fst, definitions, ret));
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    FunctionCall(FunctionCall),
    String(String),
}

impl From<FunctionCall> for Expr {
    fn from(x: FunctionCall) -> Self {
        Expr::FunctionCall(x)
    }
}

impl<T: Into<String>> From<T> for Expr {
    fn from(t: T) -> Self {
        Expr::String(t.into())
    }
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::FunctionCall(FunctionCall { name, args }) => {
                let args = args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{}({})", name, args)
            }

            Expr::String(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    name: String,
    args: Vec<Expr>,
}

impl FunctionCall {
    pub fn new(name: impl ToString) -> Self {
        FunctionCall {
            name: name.to_string(),
            args: Vec::new(),
        }
    }

    pub fn push_arg(&mut self, arg: impl Into<Expr>) {
        self.args.push(arg.into());
    }
}

impl ToString for FunctionCall {
    fn to_string(&self) -> String {
        Expr::from(self.clone()).to_string()
    }
}
