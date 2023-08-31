const { build } = require("esbuild");
const fs = require("fs");
const path = require("path");
var exec = require('child_process').exec;
const AdmZip = require("adm-zip");
const metadata = require("./src/metadata.json");

build({
  entryPoints: ['src/extension.ts'],
  outdir: 'dist',
  bundle: true,
  // Do not remove the functions `enable()`, `disable()` and `init()`
  treeShaking: false,
  // firefox60  // Since GJS 1.53.90
  // firefox68  // Since GJS 1.63.90
  // firefox78  // Since GJS 1.65.90
  // firefox91  // Since GJS 1.71.1
  // firefox102 // Since GJS 1.73.2
  target: "firefox102",
  platform: "node",
  // platform: "neutral",
  // mainFields: ['main'],
  // conditions: ['require', 'default'],
  format: 'esm',
  external: ['gi://*', 'resource:///*', 'system', 'gettext', 'cairo'],
}).then(() => {
  const metaSrc = path.resolve(__dirname, "src/metadata.json");
  const metaDist = path.resolve(__dirname, "dist/metadata.json");
  const schemaSrc = path.resolve(__dirname, "schemas");
  const schemaDist = path.resolve(__dirname, "dist/schemas");
  const dbusXmlSrc = path.resolve(__dirname, "../../bindings/dbus-xml");
  const dbusXmlDist = path.resolve(__dirname, "dist/resources/dbus");
  const zipFilename = `${metadata.uuid}.zip`;
  const zipDist = path.resolve(__dirname, zipFilename);

  exec('glib-compile-schemas schemas/',
    (error, stdout, stderr) => {
      console.log('stdout: ' + stdout);
      console.log('stderr: ' + stderr);
    });

  fs.copyFileSync(metaSrc, metaDist);

  fs.cpSync(schemaSrc, schemaDist, { recursive: true }, (err) => {
    if (err) {
      console.error(err);
    }
  });

  fs.cpSync(dbusXmlSrc, dbusXmlDist, { recursive: true }, (err) => {
    if (err) {
      console.error(err);
    }
  });

  const zip = new AdmZip();
  zip.addLocalFolder(path.resolve(__dirname, "dist"));
  zip.writeZip(zipDist);

  console.log(`Build complete. Zip file: ${zipFilename}\n`);
  console.log(`Install with: gnome-extensions install ${zipFilename}`)
  console.log(`Update with: gnome-extensions install ${zipFilename} --force`)
  console.log(`Enable with: gnome-extensions enable ${metadata.uuid} --user`)
});

