import { build } from "esbuild";
import { exec } from "child_process";
import { copyFileSync, cpSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import AdmZip from "adm-zip";
import metadata from "./src/metadata.json" assert { type: "json" };

build({
  entryPoints: ["src/extension.ts"],
  outdir: "dist",
  bundle: true,
  // Do not remove the functions `enable()`, `disable()` and `init()`
  treeShaking: false,
  // firefox60  // Since GJS 1.53.90
  // firefox68  // Since GJS 1.63.90
  // firefox78  // Since GJS 1.65.90
  // firefox91  // Since GJS 1.71.1
  // firefox102 // Since GJS 1.73.2
  target: "firefox102",
  //platform: "neutral",
  platform: "node",
  // mainFields: ['main'],
  // conditions: ['require', 'default'],
  format: "esm",
  external: ["gi://*", "resource://*", "system", "gettext", "cairo"],
}).then(() => {
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);

  const metaSrc = resolve(__dirname, "src/metadata.json");
  const metaDist = resolve(__dirname, "dist/metadata.json");
  const schemaSrc = resolve(__dirname, "schemas");
  const schemaDist = resolve(__dirname, "dist/schemas");
  const dbusXmlSrc = resolve(__dirname, "../../bindings/dbus-xml");
  const dbusXmlDist = resolve(__dirname, "dist/resources/dbus");
  const zipFilename = `${metadata.uuid}.zip`;
  const zipDist = resolve(__dirname, zipFilename);

  exec("glib-compile-schemas schemas/", (error, stdout, stderr) => {
    console.log("stdout: " + stdout);
    console.log("stderr: " + stderr);
  });

  copyFileSync(metaSrc, metaDist);

  cpSync(schemaSrc, schemaDist, { recursive: true }, (err) => {
    if (err) {
      console.error(err);
    }
  });

  cpSync(dbusXmlSrc, dbusXmlDist, { recursive: true }, (err) => {
    if (err) {
      console.error(err);
    }
  });

  const zip = new AdmZip();
  zip.addLocalFolder(resolve(__dirname, "dist"));
  zip.writeZip(zipDist);

  console.log(`Build complete. Zip file: ${zipFilename}\n`);
  console.log(`Install with: gnome-extensions install ${zipFilename}`);
  console.log(`Update with: gnome-extensions install ${zipFilename} --force`);
  console.log(`Enable with: gnome-extensions enable ${metadata.uuid} --user`);
});
