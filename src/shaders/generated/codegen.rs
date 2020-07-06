pub struct Glsl {
    uniq: usize,
    functions: Vec<String>
}

impl Glsl {
    pub fn new() -> Self {
        Glsl {
            uniq: 0,
            functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, typ: impl ToString, name: impl ToString, args: &[(impl ToString, impl ToString)]) -> Function {
        Function {
            glsl: self,
            ret: typ.to_string(),
            name: name.to_string(),
            args: args
                .into_iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            definitions: Vec::new()
        }
    }
}

impl ToString for Glsl {
    fn to_string(&self) -> String {
        self.functions.join("\n")
    }
}

pub struct Function<'a> {
    pub glsl: &'a mut Glsl,
    ret: String,
    name: String,
    args: Vec<(String, String)>,
    definitions: Vec<String>,
}

impl<'a> Function<'a> {
    pub fn gen_definition(&mut self, typ: impl ToString, expr: impl ToString) -> String {
        let ident = format!("def_{}", self.glsl.uniq);
        self.glsl.uniq += 1;

        let def = format!("{} {} = {};", typ.to_string(), ident, expr.to_string());
        self.definitions.push(def);

        ident
    }

    pub fn ret(self, expr: impl ToString) {
        let args = self.args
            .into_iter()
            .map(|(typ, name)| format!("{} {}", typ, name))
            .collect::<Vec<_>>();

        let fst = format!("{} {}({}) {{", self.ret, self.name, args.join(", "));

        let definitions = self.definitions.join("\n");

        let ret = format!("return {};", expr.to_string());

        self.glsl.functions.push(format!("{}\n{}\n{}\n}}", fst, definitions, ret));
    }
}

pub enum Expr {
    FunctionCall(FunctionCall),
    String(String),
}

impl From<FunctionCall> for Expr {
    fn from(x: FunctionCall) -> Self {
        Expr::FunctionCall(x)
    }
}

impl From<String> for Expr {
    fn from(x: String) -> Self {
        Expr::String(x)
    }
}

impl<'a> From<&'a str> for Expr {
    fn from(x: &'a str) -> Self {
        Expr::String(x.to_string())
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

pub struct FunctionCall {
    name: String,
    args: Vec<Expr>,
}

impl FunctionCall {
    pub fn new(name: impl ToString) -> Self {
        FunctionCall {
            name: name.to_string(),
            args: Vec::new()
        }
    }

    pub fn push_arg(&mut self, arg: Expr) {
        self.args.push(arg);
    }
}