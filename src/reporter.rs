use std::fmt;
use colored::*;

use error::*;

pub struct CargoBuildReporter {
    error: Error,
    colors: bool,
}

impl CargoBuildReporter {
    pub fn report(error: Error) -> Self {
        Self {
            error,
            colors: true,
        }
    }
}

impl CargoBuildReporter {
    pub fn disable_colors(&mut self) -> &mut Self {
        self.colors = false;
        self
    }
}

trait StringExt {
    fn prefix_each_line<T>(self, prefix: T) -> Self
    where
        T: ToString;

    fn prefix_each_line_hidden<T>(self, prefix: T) -> Self
    where
        T: ToString;
}

impl StringExt for String {
    fn prefix_each_line<T: ToString>(self, prefix: T) -> Self {
        let owned_prefix = prefix.to_string();
        let glue = String::from("\n") + &owned_prefix;

        owned_prefix + &self.split("\n").collect::<Vec<_>>().join(&glue)
    }

    fn prefix_each_line_hidden<T: ToString>(self, prefix: T) -> Self {
        let length = prefix.to_string().len();

        self.prefix_each_line(prefix.to_string() + "\r" + &" ".repeat(length) + "\r")
    }
}

impl fmt::Display for CargoBuildReporter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        control::set_override(self.colors);

        writeln!(
            f,
            "{}",
            self.error
                .to_string()
                .prefix_each_line("[PTX] ".bright_black())
                .prefix_each_line_hidden("cargo:warning=")
        )?;

        for next in self.error.iter().skip(1) {
            writeln!(
                f,
                "{}",
                String::from("\n caused by:")
                    .prefix_each_line("[PTX]".bright_black())
                    .prefix_each_line_hidden("cargo:warning=")
            )?;

            writeln!(
                f,
                "{}",
                next.to_string()
                    .prefix_each_line("[PTX]   ".bright_black())
                    .prefix_each_line_hidden("cargo:warning=")
            )?;
        }

        control::unset_override();
        Ok(())
    }
}
