use proc_macro::TokenStream;

fn base_values(input: TokenStream) -> (String, String, String, String, String) {
    let input_string = input.to_string();
    let mut strings = input_string.split(',');

    let value_ident = strings
        .next()
        .expect("Missing first positional argument: value identifier")
        .trim()
        .replace("\"", "");
    if value_ident.is_empty() {
        panic!("Found empty value identifier.");
    }

    let name = value_ident.replace('_', " ");

    let mut name_chars = name.chars();
    let headline = name_chars.next().unwrap().to_uppercase().chain(name_chars).collect::<String>();

    let atom_ident = format!("ident::{}", value_ident.to_uppercase());

    let atom_ident_string = strings
        .next()
        .expect("Missing second positional argument: atom ident string")
        .trim()
        .replace("\"", "");
    if atom_ident_string.is_empty() {
        panic!("Found empty atom identifier string.");
    }

    if let Some(arg) = strings.next().map(|s| s.trim()) {
        if !arg.is_empty() {
            panic!("Found unexpected third positional argument: {}.", arg);
        }
    }

    (value_ident, name, headline, atom_ident, atom_ident_string)
}

#[proc_macro]
pub fn single_string_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {hl}
impl Tag {{
    /// Returns the {n} (`{ais}`).
    pub fn {vi}(&self) -> Option<&str> {{
        self.strings_of(&{ai}).next()
    }}

    /// Removes and returns the {n} (`{ais}`).
    pub fn take_{vi}(&mut self) -> Option<String> {{
        self.take_strings_of(&{ai}).next()
    }}

    /// Sets the {n} (`{ais}`).
    pub fn set_{vi}(&mut self, {vi}: impl Into<String>) {{
        self.set_data({ai}, Data::Utf8({vi}.into()));
    }}

    /// Removes the {n} (`{ais}`).
    pub fn remove_{vi}(&mut self) {{
        self.remove_data_of(&{ai});
    }}

    /// Returns the {n} formatted in an easily readable way.
    fn format_{vi}(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{
        match self.{vi}() {{
            Some(s) => writeln!(f, \"{n}: {{}}\", s),
            None => Ok(()),
        }}
    }}
}}
    ",
        hl = headline,
        n = name,
        ais = atom_ident_string,
        vi = value_ident,
        ai = atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn multiple_string_values_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    let mut value_ident_plural = value_ident.clone();
    if value_ident_plural.ends_with('y') {
        value_ident_plural.pop();
        value_ident_plural.push_str("ies");
    } else {
        value_ident_plural.push('s');
    };

    let name_plural = value_ident_plural.replace('_', " ");

    format!(
        "
/// ### {hl}
impl Tag {{
    /// Returns all {np} (`{ais}`).
    pub fn {vip}(&self) -> impl Iterator<Item=&str> {{
        self.strings_of(&{ai})
    }}

    /// Returns the first {n} (`{ais}`).
    pub fn {vi}(&self) -> Option<&str> {{
        self.strings_of(&{ai}).next()
    }}

    /// Removes and returns all {np} (`{ais}`).
    pub fn take_{vip}(&mut self) -> impl Iterator<Item=String> + '_ {{
        self.take_strings_of(&{ai})
    }}

    /// Removes all and returns the first {n} (`{ais}`).
    pub fn take_{vi}(&mut self) -> Option<String> {{
        self.take_strings_of(&{ai}).next()
    }}

    /// Sets all {np} (`{ais}`). This will remove all other {np}.
    pub fn set_{vip}(&mut self, {vip}: impl IntoIterator<Item = String>) {{
        let data = {vip}.into_iter().map(|v| Data::Utf8(v));
        self.set_all_data({ai}, data);
    }}

    /// Sets the {n} (`{ais}`). This will remove all other {np}.
    pub fn set_{vi}(&mut self, {vi}: impl Into<String>) {{
        self.set_data({ai}, Data::Utf8({vi}.into()));
    }}

    /// Adds all {np} (`{ais}`).
    pub fn add_{vip}(&mut self, {vip}: impl IntoIterator<Item = String>) {{
        let data = {vip}.into_iter().map(|v| Data::Utf8(v));
        self.add_all_data({ai}, data);
    }}

    /// Adds an {n} (`{ais}`).
    pub fn add_{vi}(&mut self, {vi}: impl Into<String>) {{
        self.add_data({ai}, Data::Utf8({vi}.into()));
    }}

    /// Removes all {np} (`{ais}`).
    pub fn remove_{vip}(&mut self) {{
        self.remove_data_of(&{ai});
    }}

    /// Returns all {np} formatted in an easily readable way.
    fn format_{vip}(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{
        if self.{vip}().count() > 1 {{
            writeln!(f, \"{np}:\")?;
            for s in self.{vip}() {{
                writeln!(f, \"    {{}}\", s)?;
            }}
        }} else if let Some(s) = self.{vi}() {{
            writeln!(f, \"{n}: {{}}\", s)?;
        }}
        Ok(())
    }}
}}
    ",
        hl = headline,
        n = name,
        np = name_plural,
        ais = atom_ident_string,
        vi = value_ident,
        vip = value_ident_plural,
        ai = atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn flag_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {hl}
impl Tag {{
    /// Returns the {n} flag (`{ais}`).
    pub fn {vi}(&self) -> bool {{
        let vec = match self.bytes_of(&{ai}).next() {{
            Some(v) => v,
            None => return false,
        }};
        vec.get(0).map(|&v| v == 1).unwrap_or(false)
    }}

    /// Sets the {n} flag to true (`{ais}`).
    pub fn set_{vi}(&mut self) {{
        self.set_data({ai}, Data::BeSigned(vec![1u8]));
    }}

    /// Removes the {n} flag (`{ais}`).
    pub fn remove_{vi}(&mut self) {{
        self.remove_data_of(&{ai})
    }}
}}
    ",
        hl = headline,
        n = name,
        ais = atom_ident_string,
        vi = value_ident,
        ai = atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn u16_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {hl}
impl Tag {{
    /// Returns the {n} (`{ais}`)
    pub fn {vi}(&self) -> Option<u16> {{
        let vec = self.bytes_of(&{ai}).next()?;
        be_int!(vec, 0, u16)
    }}

    /// Sets the {n} (`{ais}`)
    pub fn set_{vi}(&mut self, {vi}: u16) {{
        let vec: Vec<u8> = {vi}.to_be_bytes().to_vec();
        self.set_data({ai}, Data::BeSigned(vec));
    }}

    /// Removes the {n} (`{ais}`).
    pub fn remove_{vi}(&mut self) {{
        self.remove_data_of(&{ai});
    }}
}}
    ",
        hl = headline,
        n = name,
        ais = atom_ident_string,
        vi = value_ident,
        ai = atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn u32_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {hl}
impl Tag {{
    /// Returns the {n} (`{ais}`)
    pub fn {vi}(&self) -> Option<u32> {{
        let vec = self.bytes_of(&{ai}).next()?;
        be_int!(vec, 0, u32)
    }}

    /// Sets the {n} (`{ais}`)
    pub fn set_{vi}(&mut self, {vi}: u32) {{
        let vec: Vec<u8> = {vi}.to_be_bytes().to_vec();
        self.set_data({ai}, Data::BeSigned(vec));
    }}

    /// Removes the {n} (`{ais}`).
    pub fn remove_{vi}(&mut self) {{
        self.remove_data_of(&{ai});
    }}
}}
    ",
        hl = headline,
        n = name,
        ais = atom_ident_string,
        vi = value_ident,
        ai = atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}
