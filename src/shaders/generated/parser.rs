use super::desc::*;

use nom::{
    branch::alt,
    bytes::complete as bytes,
    character::{complete as character, is_alphabetic, is_alphanumeric},
    combinator::*,
    multi::{many1, separated_list},
    sequence::{delimited, tuple},
    IResult,
};

pub fn scene(i: &[u8]) -> IResult<&[u8], Vec<Statement>> {
    all_consuming(separated_list_terminated(
        &ws(character::char(';')),
        statement,
    ))(i)
}

fn statement(i: &[u8]) -> IResult<&[u8], Statement> {
    map(
        tuple((ident, opt(args), opt(body))),
        |(name, args, body)| Statement {
            name,
            args: args.unwrap_or_default(),
            body: body.unwrap_or_default(),
        },
    )(i)
}

fn body(i: &[u8]) -> IResult<&[u8], Vec<Statement>> {
    alt((
        delimited(
            ws(character::char('{')),
            separated_list_terminated(&ws(character::char(';')), statement),
            ws(character::char('}')),
        ),
        map(statement, |stmt| vec![stmt]),
    ))(i)
}

fn complex_value(i: &[u8]) -> IResult<&[u8], String> {
    map(
        many1(ws(alt((
            simple_value,
            map(bytes::is_a("+-*/"), |b: &[u8]| {
                String::from_utf8(b.to_owned()).unwrap()
            }),
            map(args, |args| format!("({})", args.join(", "))),
        )))),
        |parts| parts.join(""),
    )(i)
}

fn simple_value(i: &[u8]) -> IResult<&[u8], String> {
    map(
        bytes::take_while1(|b| is_alphanumeric(b) || b == b'$' || b == b'.' || b == b'_' || b == b' '),
        |b: &[u8]| std::str::from_utf8(b).unwrap().trim().to_owned(),
    )(i)
}

fn ident(i: &[u8]) -> IResult<&[u8], String> {
    map(
        tuple((
            peek(verify(bytes::take(1usize), |b: &[u8]| is_alphabetic(b[0]))),
            bytes::take_while1(|b| is_alphanumeric(b) || b == b'_'),
        )),
        |(_, ident): (_, &[u8])| String::from_utf8(ident.to_owned()).unwrap(),
    )(i)
}

fn args(i: &[u8]) -> IResult<&[u8], Vec<String>> {
    delimited(
        ws(character::char('(')),
        separated_list(ws(character::char(',')), complex_value),
        ws(character::char(')')),
    )(i)
}

fn separated_list_terminated<'a, 'b, O1: 'b, O2: 'b, E: 'b, P1: 'b, P2: 'b>(
    sep: &'b P1,
    parser: P2,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<O2>, E> + 'b
where
    E: nom::error::ParseError<&'a [u8]>,
    P1: Fn(&'a [u8]) -> IResult<&'a [u8], O1, E>,
    P2: Fn(&'a [u8]) -> IResult<&'a [u8], O2, E>,
    'a: 'b,
{
    map(tuple((separated_list(sep, parser), opt(sep))), |(a, _)| a)
}

fn ws<'a, O, E, P>(parser: P) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    E: nom::error::ParseError<&'a [u8]>,
    P: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(
        bytes::take_while(|b| b == b' ' || b == b'\t' || b == b'\n'),
        parser,
        bytes::take_while(|b| b == b' ' || b == b'\t' || b == b'\n'),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ident() {
        assert_eq!(ident(b"abcde").unwrap().1, "abcde");
        assert_eq!(ident(b"hello1").unwrap().1, "hello1");
        assert_eq!(ident(b"hell_o").unwrap().1, "hell_o");
        assert!(ident(b"12a").is_err());
    }

    #[test]
    fn test_simple_value() {
        assert_eq!(simple_value(b"1.0").unwrap().1, "1.0");
        assert_eq!(simple_value(b"hello").unwrap().1, "hello");
        assert!(simple_value(b"()").is_err());
    }

    #[test]
    fn test_complex_value() {
        assert_eq!(complex_value(b"vec3(1,2,3)").unwrap().1, "vec3(1, 2, 3)");
        assert_eq!(
            complex_value(b"vec3(1,2,vec2(1))").unwrap().1,
            "vec3(1, 2, vec2(1))"
        );
        assert_eq!(
            complex_value(b"vec3(1, 2, vec2(1))").unwrap().1,
            "vec3(1, 2, vec2(1))"
        );
        assert_eq!(
            complex_value(b"vec3(1 , 2 , vec2(1))").unwrap().1,
            "vec3(1, 2, vec2(1))"
        );
    }

    #[test]
    fn test_args() {
        assert_eq!(
            args(b"(1,hello,vec3(5))").unwrap().1,
            &["1", "hello", "vec3(5)"]
        );
        assert_eq!(
            args(b"(1, hello, vec3(5))").unwrap().1,
            &["1", "hello", "vec3(5)"]
        );
    }

    #[test]
    fn test_body() {
        assert_eq!(body(b"{}").unwrap().1.len(), 0);

        let body = body(b"{ hello() {  } }").unwrap().1;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0].to_string(), "hello(){}");
    }

    #[test]
    fn test_statement() {
        let stmt = statement(b"hello()").unwrap().1;
        assert_eq!(stmt.to_string(), "hello(){}");

        let stmt = statement(b"hello(1, 2, vec3(1))").unwrap().1;
        assert_eq!(stmt.to_string(), "hello(1, 2, vec3(1)){}");

        let stmt = all_consuming(statement)(b"hello(){world()}").unwrap().1;
        assert_eq!(stmt.to_string(), "hello(){world(){}}");

        let s = r#"
        hello() {
            world();
            at(1, 2, 3) { cube() }
        }
        "#
        .trim();
        let stmt = all_consuming(statement)(s.as_bytes()).unwrap().1;
        assert_eq!(
            stmt.to_string(),
            "hello(){world(){}; at(1, 2, 3){cube(){}}}"
        );

        let s = r#"
        at(1,2,3) scale(4,5,6) { cube() }
        "#
        .trim();
        let stmt = all_consuming(statement)(s.as_bytes()).unwrap().1;
        assert_eq!(stmt.to_string(), "at(1, 2, 3){scale(4, 5, 6){cube(){}}}");

        let stmt = statement(b"union { cube(); sphere(); }").unwrap().1;
        assert_eq!(stmt.to_string(), "union(){cube(){}; sphere(){}}");
    }
}
