#!/bin/bash

## Script to initialise dev-environment (types)
gv="44"
wd=${PWD}

# cleanup
rm -rf @types

# generate GJS from gir (this does not include the extensions)
echo "Generating GJS types from gir.."
npx ts-for-gir generate Shell-12 St-12 Gtk-4.0 \
  -g /usr/share/gir-1.0 \
  -g /usr/share/gnome-shell \
  -g /usr/share/gnome-shell/gir-1.0 \
  -g /usr/lib64/mutter-12 \
  -t esm -o @types/Gjs

# get latest js (44) in this case and create the types for it
echo "Generating GJS Extension (Gex) types from extension source.."
mkdir -p ./_tmp/
cd ./_tmp
wget -q -O gnome-shell-js-${gv}.tar.gz https://gitlab.gnome.org/GNOME/gnome-shell/-/archive/gnome-${gv}/gnome-shell-gnome-${gv}.tar.gz?path=js
tar xf gnome-shell-js-${gv}.tar.gz
cd gnome-shell-gnome-${gv}-js
cat >tsconfig.json <<EOL
{
  "include": ["js/ui/*"],
  "exclude": [
    "js/ui/shellDBus.js",
    "node_modules",
    "**/node_modules/*"
  ],
  "compilerOptions": {
    "allowJs": true,
    "declaration": true,
    "emitDeclarationOnly": true,
    "outDir": "gex-types",
    "declarationMap": true,
    "lib": ["es2019"]
  }
}
EOL
npx tsc
cd ${wd}
mv ./_tmp/gnome-shell-gnome-${gv}-js/gex-types @types/Gex
# rm -rf ./_tmp/

echo "done."

exit 0