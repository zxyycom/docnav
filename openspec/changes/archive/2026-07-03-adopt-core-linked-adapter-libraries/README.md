# adopt-core-linked-adapter-libraries

Make default document operation adapter implementations come from adapter-layer workspace crates shipped with the `docnav` core release and registered in one static core registry. This change removes the default runtime adapter package/registration model while preserving adapter-owned parsing, ref, navigation, pagination, and native option boundaries.
