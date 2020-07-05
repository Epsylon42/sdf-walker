use std::fmt::Display;

pub struct Glsl {
    uniq: usize,
    lines: Vec<String>,
}

impl Glsl {
    pub fn new() -> Self {
        Glsl {
            uniq: 0,
            lines: Vec::new(),
        }
    }

    pub fn add_function(
        &mut self,
        ret: impl Into<String>,
        name: impl Into<String>,
        args: &[(impl ToString, impl ToString)],
    ) -> Function {
        Function {
            glsl: self,
            ret: ret.into(),
            name: name.into(),
            args: args
                .into_iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            body: Vec::new(),
        }
    }
}

impl ToString for Glsl {
    fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}

pub struct Function<'a> {
    pub glsl: &'a mut Glsl,
    ret: String,
    name: String,
    args: Vec<(String, String)>,
    body: Vec<String>,
}

impl<'a> Function<'a> {
    pub fn gen_definition(&mut self, typ: impl Display, value: impl Display) -> String {
        let name = format!("def_{}", self.glsl.uniq);
        self.glsl.uniq += 1;

        self.body.push(format!("{} {} = {}", typ, name, value));

        name
    }

    pub fn ret(mut self, value: impl Display) {
        self.body.push(format!("return {}", value));
        self.glsl.lines.push(self.to_string());
    }
}

impl<'a> ToString for Function<'a> {
    fn to_string(&self) -> String {
        let Function {
            ret,
            name,
            args,
            body,
            ..
        } = self;

        let args = args
            .iter()
            .map(|(typ, val)| format!("{} {}", typ, val))
            .collect::<Vec<_>>();

        format!(
            "{} {}({}) {{ {}; }}",
            ret,
            name,
            args.join(","),
            body.join(";")
        )
    }
}
