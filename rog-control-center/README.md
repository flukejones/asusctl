# ROGALOG

### Translations

You can help with translations by following https://slint.dev/releases/1.1.0/docs/slint/src/concepts/translations#translate-the-strings

Begin by copying `rog-control-center/translations/en/rog-control-center.po` to `rog-control-center/translations/<YOUR LOCALE>/rog-control-center.po`, then edit that file.

Run `msgfmt rog-control-center/translations/<YOUR LOCALE>/rog-control-center.po -o rog-control-center/translations/<YOUR LOCALE>/LC_MESSAGES/rog-control-center.mo` to make the binary formatted translation where `<YOUR LOCALE>` is changed to your translation locale.

To test you local translations run `RUST_TRANSLATIONS=1 rog-control-center`.