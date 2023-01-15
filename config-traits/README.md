# config-traits

`config_traits` is a crate that broke out from the requirement to manage various
different config files, including parsing from different formats and updating
them from previous versions where fields or names are changed in some way.

The end canonical file format is `.ron` as this supports rust types well, and includes
the ability to add commenting, and is less verbose than `json`. Currently the crate will
also try to parse from `json` and `toml` if the `ron` parsing fails, then update to `ron`
format.