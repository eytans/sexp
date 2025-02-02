//! A lightweight, self-contained s-expression parser and data format.
//! Use `parse` to get an s-expression from its string representation, and the
//! `Display` trait to serialize it, potentially by doing `sexp.to_string()`.

#![deny(missing_docs)]
#![deny(unsafe_code)]

use std::borrow::Cow;
use std::cmp;
use std::collections::BTreeMap;
use std::error;
use std::fmt;
use std::str::{self, FromStr};

/// A single data element in an s-expression. Floats are excluded to ensure
/// atoms may be used as keys in ordered and hashed data structures.
///
/// All strings must be valid utf-8.
#[derive(PartialEq, Clone, PartialOrd)]
#[allow(missing_docs)]
pub enum Atom {
  S(String),
  I(i64),
  F(f64),
}


impl Atom {
  /// Returns true if this atom is a string.
  pub fn is_string(&self) -> bool {
    match self {
      &Atom::S(_) => true,
      _           => false,
    }
  }

  /// Returns true if this atom is an integer.
  pub fn is_int(&self) -> bool {
    match self {
      &Atom::I(_) => true,
      _           => false,
    }
  }

  /// Returns true if this atom is a float.
  pub fn is_float(&self) -> bool {
    match self {
      &Atom::F(_) => true,
      _           => false,
    }
  }

  /// Return the string contained in this atom, panic if it is not a string.
  pub fn string(&self) -> &str {
    self.try_string().expect("not a string")
  }

  /// Try to return the string contained in this atom, or None if it is not a
  /// string.
  pub fn try_string(&self) -> Option<&str> {
    match self {
      &Atom::S(ref s) => Some(s),
      _               => None,
    }
  }

  /// Consume this atom and return its string, or None if it is not a string.
  pub fn into_string(self) -> Option<String> {
    match self {
      Atom::S(s) => Some(s),
      _          => None,
    }
  }

  /// Return the integer contained in this atom, panic if it is not an integer.
  pub fn int(&self) -> i64 {
    self.try_int().expect("not an int")
  }

  /// Try to return the integer contained in this atom, or None if it is not an
  /// integer.
  pub fn try_int(&self) -> Option<i64> {
    match self {
      &Atom::I(i) => Some(i),
      _           => None,
    }
  }

  /// Consume this atom and return its integer, or None if it is not an
  /// integer.
  pub fn into_int(self) -> Option<i64> {
    match self {
      Atom::I(i) => Some(i),
      _          => None,
    }
  }

  /// Return the float contained in this atom, panic if it is not a float.
  pub fn float(&self) -> f64 {
    self.try_float().expect("not a float")
  }

  /// Try to return the float contained in this atom, or None if it is not a
  /// float.
  pub fn try_float(&self) -> Option<f64> {
    match self {
      &Atom::F(f) => Some(f),
      _           => None,
    }
  }

  /// Consume this atom and return its float, or None if it is not a float.
  pub fn into_float(self) -> Option<f64> {
    match self {
      Atom::F(f) => Some(f),
      _          => None,
    }
  }
}


/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(PartialEq, Clone, PartialOrd)]
#[allow(missing_docs)]
pub enum Sexp {
  Atom(Atom),
  List(Vec<Sexp>),
}

impl Sexp {
  /// Returns true if this s-expression is an atom.
  pub fn is_atom(&self) -> bool {
    match self {
      Sexp::Atom(_) => true,
      _             => false,
    }
  }

  /// Returns true if this s-expression is a list.
  pub fn is_list(&self) -> bool {
    match *self {
      Sexp::List(_) => true,
      _             => false,
    }
  }

  /// Return the atom contained in this s-expression, panic if it is a list.
  pub fn atom(&self) -> &Atom {
    self.try_atom().expect(&format!("Expecting an atom, got: {}", self))
  }

  /// Try to return the atom contained in this s-expression, or None if it is a
  pub fn try_atom(&self) -> Option<&Atom> {
    match self {
      &Sexp::Atom(ref a) => Some(a),
      _                  => None,
    }
  }

  /// Consume this s-expression and return its atom, or None if it is a list.
  pub fn into_atom(self) -> Option<Atom> {
    match self {
      Sexp::Atom(a) => Some(a),
      _             => None,
    }
  }

  /// Return the list contained in this s-expression, panic if it is an atom.
  pub fn list(&self) -> &Vec<Sexp> {
    self.try_list().expect(&format!("Expecting a list, got: {}", self))
  }

  /// Try to return the list contained in this s-expression, or None if it is an
  /// atom.
  pub fn try_list(&self) -> Option<&Vec<Sexp>> {
    match self {
      &Sexp::List(ref l) => Some(l),
      _                  => None,
    }
  }

  /// Consume this s-expression and return its list, or None if it is an atom.
  pub fn into_list(self) -> Option<Vec<Sexp>> {
    match self {
      Sexp::List(l) => Some(l),
      _             => None,
    }
  }

  /// Turn s-expression list into a map from key value pairs.
  pub fn into_map(self) -> Option<BTreeMap<String, Sexp>> {
    match self {
      Sexp::List(l) => {
        let mut map = BTreeMap::new();
        for sub_l in l.into_iter() {
          assert!(sub_l.is_list() && sub_l.list().len() == 2, 
            "Assertion to map failed (is_list {} len {:?}) on: {}", 
            sub_l.is_list(), 
            sub_l.try_list().map(|x| x.len()), 
            sub_l.to_string().chars().take(100).collect::<String>()
          );
          let mut sub_l = sub_l.into_list().unwrap();
          let value = sub_l.remove(1);
          let key = sub_l.remove(0);
          let key = key.into_atom()?.into_string()?;
          assert!(!map.contains_key(&key));
          map.insert(key, value);
        }
        Some(map)
      },
      _ => None,
    }
  }
}

#[test]
fn sexp_size() {
  // I just want to see when this changes, in the diff.
  use std::mem;
  assert_eq!(mem::size_of::<Sexp>(), mem::size_of::<isize>()*5);
}

/// The representation of an s-expression parse error.
pub struct Error {
  /// The error message.
  pub message: &'static str,
  /// The line number on which the error occurred.
  pub line:    usize,
  /// The column number on which the error occurred.
  pub column:  usize,
  /// The index in the given string which caused the error.
  pub index:   usize,
}

impl error::Error for Error {
  fn description(&self) -> &str { self.message }
  fn cause(&self) -> Option<&dyn error::Error> { None }
}

/// Since errors are the uncommon case, they're boxed. This keeps the size of
/// structs down, which helps performance in the common case.
///
/// For example, an `ERes<()>` becomes 8 bytes, instead of the 24 bytes it would
/// be if `Err` were unboxed.
type Err = Box<Error>;

/// Helps clean up type signatures, but shouldn't be exposed to the outside
/// world.
type ERes<T> = Result<T, Err>;

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}:{}: {}", self.line, self.column, self.message)
  }
}

impl fmt::Debug for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self)
  }
}

#[test]
fn show_an_error() {
  assert_eq!(format!("{:?}", parse("(aaaa").unwrap_err()), "1:4: unexpected eof");
}

fn get_line_and_column(s: &str, pos: usize) -> (usize, usize) {
  let mut line: usize = 1;
  let mut col:  isize = -1;
  for c in s.chars().take(pos+1) {
    if c == '\n' {
      line += 1;
      col   = -1;
    } else {
      col  += 1;
    }
  }
  (line, cmp::max(col, 0) as usize)
}

#[test]
fn line_and_col_test() {
  let s = "0123456789\n0123456789\n\n6";
  assert_eq!(get_line_and_column(s, 4), (1, 4));

  assert_eq!(get_line_and_column(s, 10), (2, 0));
  assert_eq!(get_line_and_column(s, 11), (2, 0));
  assert_eq!(get_line_and_column(s, 15), (2, 4));

  assert_eq!(get_line_and_column(s, 21), (3, 0));
  assert_eq!(get_line_and_column(s, 22), (4, 0));
  assert_eq!(get_line_and_column(s, 23), (4, 0));
  assert_eq!(get_line_and_column(s, 500), (4, 0));
}

#[cold]
fn err_impl(message: &'static str, s: &str, pos: &usize) -> Err {
  let (line, column) = get_line_and_column(s, *pos);
  Box::new(Error {
    message: message,
    line:    line,
    column:  column,
    index:   *pos,
  })
}

fn err<T>(message: &'static str, s: &str, pos: &usize) -> ERes<T> {
  Err(err_impl(message, s, pos))
}

/// A helpful utility to trace the execution of a parser while testing.  It will
/// be compiled out in release builds.
#[allow(unused_variables)]
fn dbg(msg: &str, pos: &usize) {
  //println!("{} @ {}", msg, pos)
}

fn atom_of_string(s: String) -> Atom {
  match FromStr::from_str(&s) {
    Ok(i)  => return Atom::I(i),
    Err(_) => {},
  };

  match FromStr::from_str(&s) {
    Ok(f) => return Atom::F(f),
    Err(_) => {},
  };

  Atom::S(s)
}

// returns the char it found, and the new size if you wish to consume that char
fn peek(s: &str, pos: &usize) -> ERes<(char, usize)> {
  dbg("peek", pos);
  if *pos == s.len() { return err("unexpected eof", s, pos) }
  if s.is_char_boundary(*pos) {
    let ch = s[*pos..].chars().next().unwrap();
    let next = *pos + ch.len_utf8();
    Ok((ch, next))
  } else {
    // strings must be composed of valid utf-8 chars.
    unreachable!()
  }
}

fn expect(s: &str, pos: &mut usize, c: char) -> ERes<()> {
  dbg("expect", pos);
  let (ch, next) = peek(s, pos)?;
  *pos = next;
  if ch == c { Ok(()) } else { err("unexpected character", s, pos) }
}

fn consume_until_newline(s: &str, pos: &mut usize) -> ERes<()> {
  loop {
    if *pos == s.len() { return Ok(()) }
    let (ch, next) = peek(s, pos)?;
    *pos = next;
    if ch == '\n' { return Ok(()) }
  }
}

// zero or more spaces
fn zspace(s: &str, pos: &mut usize) -> ERes<()> {
  dbg("zspace", pos);
  loop {
    if *pos == s.len() { return Ok(()) }
    let (ch, next) = peek(s, pos)?;

    if ch == ';'               { consume_until_newline(s, pos)? }
    else if ch.is_whitespace() { *pos = next; }
    else                       { return Ok(()) }
  }
}

fn parse_quoted_atom(s: &str, pos: &mut usize) -> ERes<Atom> {
  dbg("parse_quoted_atom", pos);
  let mut cs: String = String::new();

  expect(s, pos, '"')?;

  loop {
    let (ch, next) = peek(s, pos)?;
    if ch == '"' {
      *pos = next;
      break;
    } else if ch == '\\' {
      let (postslash, nextnext) = peek(s, &next)?;
      if postslash == '"' || postslash == '\\' {
        cs.push(postslash);
      } else {
        cs.push(ch);
        cs.push(postslash);
      }
      *pos = nextnext;
    } else {
      cs.push(ch);
      *pos = next;
    }
  }

  // Do not try i64 conversion, since this atom was explicitly quoted.
  Ok(Atom::S(cs))
}

fn parse_unquoted_atom(s: &str, pos: &mut usize) -> ERes<Atom> {
  dbg("parse_unquoted_atom", pos);
  let mut cs: String = String::new();

  loop {
    if *pos == s.len() { break }
    let (c, next) = peek(s, pos)?;

    if c == ';' { consume_until_newline(s, pos)?; break }
    if c.is_whitespace() || c == '(' || c == ')' { break }
    cs.push(c);
    *pos = next;
  }

  Ok(atom_of_string(cs))
}

fn parse_atom(s: &str, pos: &mut usize) -> ERes<Atom> {
  dbg("parse_atom", pos);
  let (ch, _) = peek(s, pos)?;

  if ch == '"' { parse_quoted_atom  (s, pos) }
  else         { parse_unquoted_atom(s, pos) }
}

fn parse_list(s: &str, pos: &mut usize) -> ERes<Vec<Sexp>> {
  dbg("parse_list", pos);
  zspace(s, pos)?;
  expect(s, pos, '(')?;

  let mut sexps: Vec<Sexp> = Vec::new();

  loop {
    zspace(s, pos)?;
    let (c, next) = peek(s, pos)?;
    if c == ')' {
      *pos = next;
      break;
    }
    sexps.push(parse_sexp(s, pos)?);
  }

  zspace(s, pos)?;

  Ok(sexps)
}

fn parse_sexp(s: &str, pos: &mut usize) -> ERes<Sexp> {
  dbg("parse_sexp", pos);
  zspace(s, pos)?;
  let (c, _) = peek(s, pos)?;
  let r =
    if c == '(' { Ok(Sexp::List(parse_list(s, pos)?)) }
    else        { Ok(Sexp::Atom(parse_atom(s, pos)?)) };
  zspace(s, pos)?;
  r
}

/// Constructs an atomic s-expression from a string.
pub fn atom_s(s: &str) -> Sexp {
  Sexp::Atom(Atom::S(s.to_owned()))
}

/// Constructs an atomic s-expression from an int.
pub fn atom_i(i: i64) -> Sexp {
  Sexp::Atom(Atom::I(i))
}

/// Constructs an atomic s-expression from a float.
pub fn atom_f(f: f64) -> Sexp {
  Sexp::Atom(Atom::F(f))
}

/// Constructs a list s-expression given a slice of s-expressions.
pub fn list(xs: &[Sexp]) -> Sexp {
  Sexp::List(xs.to_owned())
}

/// Reads an s-expression out of a `&str`.
#[inline(never)]
pub fn parse(s: &str) -> Result<Sexp, Box<Error>> {
  let mut pos = 0;
  let ret = parse_sexp(s, &mut pos)?;
  if pos == s.len() { Ok(ret) } else { err("unrecognized post-s-expression data", s, &pos) }
}

// TODO: Pretty print in lisp convention, instead of all on the same line,
// packed as tightly as possible. It's kinda ugly.

fn is_num_string(s: &str) -> bool {
  let x: Result<i64, _> = FromStr::from_str(&s);
  let y: Result<f64, _> = FromStr::from_str(&s);
  x.is_ok() || y.is_ok()
}

fn string_contains_whitespace(s: &str) -> bool {
  for c in s.chars() {
    if c.is_whitespace() { return true }
  }
  false
}

fn quote(s: &str) -> Cow<str> {
  if !s.contains("\"")
  && !string_contains_whitespace(s)
  && !is_num_string(s) {
    Cow::Borrowed(s)
  } else {
    let mut r: String = "\"".to_string();
    r.push_str(&s.replace("\\", "\\\\").replace("\"", "\\\""));
    r.push_str("\"");
    Cow::Owned(r)
  }
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Atom::S(ref s) => write!(f, "{}", quote(s)),
      Atom::I(i)     => write!(f, "{}", i),
      Atom::F(d)     => write!(f, "{}", d),
    }
  }
}

impl fmt::Display for Sexp {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Sexp::Atom(ref a) => write!(f, "{}", a),
      Sexp::List(ref xs) => {
        write!(f, "(")?;
        for (i, x) in xs.iter().enumerate() {
          let s = if i == 0 { "" } else { " " };
          write!(f, "{}{}", s, x)?;
        }
        write!(f, ")")
      },
    }
  }
}

impl fmt::Debug for Atom {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self)
  }
}

impl fmt::Debug for Sexp {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self)
  }
}

#[test]
fn test_hello_world() {
  assert_eq!(
    parse("(hello -42\n\t  -4.0 \"world\") ; comment").unwrap(),
    list(&[ atom_s("hello"), atom_i(-42), atom_f(-4.0), atom_s("world") ]));
}

#[test]
fn test_escaping() {
  assert_eq!(
    parse("(\"\\\"\\q\" \"1234\" 1234)").unwrap(),
    list(&[ atom_s("\"\\q"), atom_s("1234"), atom_i(1234) ]));
}

#[test]
fn test_pp() {
  let s = "(hello world (what is (up) (4 6.4 you \"123\\\\ \\\"\")))";
  let sexp = parse(s).unwrap();
  assert_eq!(s, sexp.to_string());
  assert_eq!(s, format!("{:?}", sexp));
}

#[test]
fn test_tight_parens() {
    let s = "(hello(world))";
    let sexp = parse(s).unwrap();
    assert_eq!(sexp, Sexp::List(vec![Sexp::Atom(Atom::S("hello".into())),
                                     Sexp::List(vec![Sexp::Atom(Atom::S("world".into()))])]));
    let s = "(this (has)tight(parens))";
    let s2 = "( this ( has ) tight ( parens ) )";
    assert_eq!(parse(s).unwrap(), parse(s2).unwrap());
}

#[test]
fn test_space_in_atom() {
  let sexp = list(&[ atom_s("hello world")]);
  let sexp_as_string = sexp.to_string();
  assert_eq!("(\"hello world\")", sexp_as_string);
  assert_eq!(sexp, parse(&sexp_as_string).unwrap());
}
