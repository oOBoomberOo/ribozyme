use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::sequence::tuple;
use nom::IResult;

pub fn parse_namespace(input: &str) -> IResult<&str, (&str, &str)> {
	alt((full_namespace, partial_namespace))(input)
}

fn partial_namespace(input: &str) -> IResult<&str, (&str, &str)> {
	let (rest, path) = namespace_path(input)?;
	Ok((rest, ("minecraft", path)))
}

fn full_namespace(input: &str) -> IResult<&str, (&str, &str)> {
	let (rest, (header, _, path)) = tuple((namespace, char(':'), namespace_path))(input)?;
	Ok((rest, (header, path)))
}

fn is_namespace(c: char) -> bool {
	c.is_ascii_digit() || c.is_ascii_lowercase() || c == '.' || c == '_' || c == '#'
}

fn is_namespace_path(c: char) -> bool {
	is_namespace(c) || c == '/'
}

fn namespace(input: &str) -> IResult<&str, &str> {
	take_while1(is_namespace)(input)
}

fn namespace_path(input: &str) -> IResult<&str, &str> {
	take_while1(is_namespace_path)(input)
}
