use std::fmt::Write as fmt_Write;
use std::fmt;

use Message;
use reflect::ReflectFieldRef;
use reflect::ReflectValueRef;
use reflect::ReflectRepeatedRef;
use json::float;
use std::f32;
use std::f64;
use reflect::ReflectMapRef;
use json::base64;

struct Printer {
    buf: String,
}

trait JsonFloat : fmt::Display + fmt::Debug {
    fn is_nan(&self) -> bool;
    fn is_pos_infinity(&self) -> bool;
    fn is_neg_infinity(&self) -> bool;

    fn write_to_json(&self, w: &mut String) -> fmt::Result {
        if self.is_nan() {
            write!(w, "\"{}\"", float::PROTOBUF_JSON_NAN)
        } else if self.is_pos_infinity() {
            write!(w, "\"{}\"", float::PROTOBUF_JSON_INF)
        } else if self.is_neg_infinity() {
            write!(w, "\"{}\"", float::PROTOBUF_JSON_MINUS_INF)
        } else {
            write!(w, "{:?}", self)
        }
    }
}

impl JsonFloat for f32 {
    fn is_nan(&self) -> bool {
        f32::is_nan(*self)
    }

    fn is_pos_infinity(&self) -> bool {
        f32::is_infinite(*self) && self > &0.0
    }

    fn is_neg_infinity(&self) -> bool {
        f32::is_infinite(*self) && self < &0.0
    }
}

impl JsonFloat for f64 {
    fn is_nan(&self) -> bool {
        f64::is_nan(*self)
    }

    fn is_pos_infinity(&self) -> bool {
        f64::is_infinite(*self) && self > &0.0
    }

    fn is_neg_infinity(&self) -> bool {
        f64::is_infinite(*self) && self < &0.0
    }
}

impl Printer {
    fn print_comma_but_first(&mut self, first: &mut bool) -> fmt::Result {
        if *first {
            *first = false;
            Ok(())
        } else {
            write!(self.buf, ", ")
        }
    }

    fn print_json_string(&mut self, value: &str) -> fmt::Result {
        write!(self.buf, "\"")?;
        for c in value.chars() {
            match c {
                '"' => write!(self.buf, "\\\""),
                '\\' => write!(self.buf, "\\\\"),
                '\n' => write!(self.buf, "\\n"),
                '\r' => write!(self.buf, "\\r"),
                '\t' => write!(self.buf, "\\t"),
                c if c.is_control() => write!(self.buf, "\\u{:04x}", c as u32),
                c => write!(self.buf, "{}", c),
            }?;
        }
        write!(self.buf, "\"")?;
        Ok(())
    }

    fn print_value(&mut self, value: &ReflectValueRef) -> fmt::Result {
        match value {
            ReflectValueRef::U32(v) => write!(self.buf, "{}", v),
            ReflectValueRef::U64(v) => write!(self.buf, "\"{}\"", v),
            ReflectValueRef::I32(v) => write!(self.buf, "{}", v),
            ReflectValueRef::I64(v) => write!(self.buf, "\"{}\"", v),
            ReflectValueRef::F32(v) => v.write_to_json(&mut self.buf),
            ReflectValueRef::F64(v) => v.write_to_json(&mut self.buf),
            ReflectValueRef::Bool(v) => write!(self.buf, "{}", v),
            ReflectValueRef::String(v) => self.print_json_string(v),
            ReflectValueRef::Bytes(v) => {
                let encoded = base64::encode(&v);
                self.print_json_string(&encoded)
            }
            // TODO: option to output JSON as number
            ReflectValueRef::Enum(v) => write!(self.buf, "\"{}\"", v.name()),
            ReflectValueRef::Message(v) => self.print_message(*v),
        }
    }

    fn print_repeated(&mut self, repeated: &ReflectRepeatedRef) -> fmt::Result {
        write!(self.buf, "[")?;
        for (i, item) in repeated.into_iter().enumerate() {
            if i != 0 {
                write!(self.buf, ", ")?;
            }
            self.print_value(&item)?;
        }
        write!(self.buf, "]")?;
        Ok(())
    }

    fn print_map(&mut self, map: &ReflectMapRef) -> fmt::Result {
        write!(self.buf, "{{")?;
        for (i, (k, v)) in map.into_iter().enumerate() {
            if i != 0 {
                write!(self.buf, ", ")?;
            }
            self.print_value(&k)?;
            write!(self.buf, ": ")?;
            self.print_value(&v)?;
        }
        write!(self.buf, "}}")?;
        Ok(())
    }

    fn print_message(&mut self, message: &Message) -> fmt::Result {
        let descriptor = message.descriptor();

        write!(self.buf, "{{")?;

        let mut first = true;

        for field in descriptor.fields() {
            match field.get_reflect(message) {
                ReflectFieldRef::Optional(None) => {}
                ReflectFieldRef::Optional(Some(v)) => {
                    self.print_comma_but_first(&mut first)?;
                    write!(self.buf, "{}: ", field.json_name())?;
                    self.print_value(&v)?;
                }
                ReflectFieldRef::Repeated(v) => {
                    if !v.is_empty() {
                        write!(self.buf, "{}: ", field.json_name())?;
                        self.print_repeated(&v)?;
                    }
                }
                ReflectFieldRef::Map(v) => {
                    if !v.is_empty() {
                        write!(self.buf, "{}: ", field.json_name())?;
                        self.print_map(&v)?;
                    }
                }
            }
        }

        write!(self.buf, "}}")?;
        Ok(())
    }
}

pub fn print_to_string(message: &Message) -> String {
    let mut printer = Printer {
        buf: String::new()
    };
    printer.print_message(message).unwrap();
    printer.buf
}
