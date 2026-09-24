# IES files

The editor opens `.ies` files through Open, drag and drop, and the operating system's file association. Save writes to the current file format. Save As accepts `.ldt` and `.ies`. Export IES writes a separate copy and leaves the open document and its unsaved state in place.

IES import accepts LM-63 1986, 1991, 1995, 2002, and 2019 headers with Type C photometry and `TILT=NONE`. The horizontal grid may cover 0 to 90, 0 to 180, or 0 to 360 degrees. Unsupported tilt tables and Type A or B photometry produce an error. Import converts candela to the editor's cd/klm model and applies the IES candela multiplier and ballast factors. An absolute IES file uses a 1000 lm reference basis in the editor.

IES export uses LM-63-2002 Type C absolute photometry (`-1` lumens per lamp). It converts the current cd/klm grid and lamp flux back to candela. The exported file carries the luminaire name and manufacturer field. Other IES keywords have no matching fields in the EULUMDAT editor and are not retained when an imported IES file is saved.

IES width, length, and height describe the luminous area. Import leaves the EULUMDAT luminaire housing dimensions at zero because IES does not provide them. Enter housing dimensions before saving an imported file as LDT.
