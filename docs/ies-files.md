# IES files

The editor opens `.ies` files through Open, drag and drop, and the operating system's file association. Save writes to the current file format. Save As accepts `.ldt` and `.ies`. Export IES writes a separate copy and leaves the open document and its unsaved state in place.

IES import accepts LM-63 1986, 1991, 1995, 2002, and 2019 headers with Type C photometry and `TILT=NONE`. The horizontal grid may cover 0 to 90, 0 to 180, or 0 to 360 degrees. Unsupported tilt tables and Type A or B photometry produce an error. Import converts candela to the editor's cd/klm model and applies the IES candela multiplier and ballast factors. Relative files derive their light-output ratio from the measured distribution and declared lamp lumens. Absolute files use the integrated luminaire output as their flux basis and have a 100% light-output ratio.

IES export uses LM-63-2002 Type C absolute photometry (`-1` lumens per lamp). It converts the current cd/klm grid and the first lamp set's flux back to candela, and uses that set's wattage. The exported file carries the luminaire name and manufacturer field. Other IES keywords have no matching fields in the EULUMDAT editor and are not retained when an imported IES file is saved.

IES width, length, and height describe the luminous area. Import translates circular luminous dimensions to EULUMDAT diameter and zero width; shapes without an EULUMDAT equivalent produce an error. Import leaves the EULUMDAT luminaire housing dimensions at zero because IES does not provide them. Enter housing dimensions before saving an imported file as LDT.

Import accepts UTF-8 and Windows-1252 metadata. Full-circle files retain their C360 row, even if its measured values differ from C0. Export adds C360 only when the model does not already contain it.

For compatibility checks with downloaded manufacturer files, place supported `.ies` samples in a local directory and run `IES_QA_DIR=/path/to/samples cargo test external_ies_files_keep_candela_after_export -- --ignored` from `src-tauri/`. The external samples used during development came from the [eulumdat-rs test files](https://github.com/holg/eulumdat-rs/tree/main/tests/files); they are not bundled in this repository.
