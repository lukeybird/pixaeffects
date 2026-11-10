export type PixaProduct = { id: string; name: string; category: string; use: string; summary: string; description: string; capabilities: [string,string][] };
export const products: PixaProduct[] = [
  {
    "id": "pixashop",
    "name": "PixaShop",
    "category": "Images",
    "use": "image editing",
    "summary": "Edit photos, paint and compose with layers.",
    "description": "Bring images together in a dedicated desktop editor with layers, masks, brushes and familiar image-editing workflows.",
    "capabilities": [
      [
        "Layers and masks",
        "Build compositions with separate layers and control where each edit appears."
      ],
      [
        "Brushes and selections",
        "Paint, select and refine specific parts of an image."
      ],
      [
        "Desktop workspace",
        "Open your files locally and follow the PixaShop manual for supported workflows."
      ]
    ]
  },
  {
    "id": "pixaeffects",
    "name": "PixaEffects",
    "category": "Motion",
    "use": "motion graphics",
    "summary": "Animate titles, build compositions and explore visual effects.",
    "description": "A desktop motion workspace for layered compositions, keyframes and effects. This Pixa edition is still being prepared for release.",
    "capabilities": [
      [
        "Layered compositions",
        "Arrange footage and graphic elements in a composition."
      ],
      [
        "Animation",
        "Shape movement over time with keyframes."
      ],
      [
        "Visual effects",
        "Explore effects within a focused motion workspace."
      ]
    ]
  },
  {
    "id": "pixavector",
    "name": "PixaVector",
    "category": "Design",
    "use": "vector illustration",
    "summary": "Give lines, shapes and ideas a sharper edge.",
    "description": "A vector workspace for illustrations and scalable graphics, prepared as part of the Pixa development family.",
    "capabilities": [
      [
        "Paths and shapes",
        "Construct artwork from editable vector geometry."
      ],
      [
        "Illustration",
        "Combine shapes, color and typography in a single canvas."
      ],
      [
        "Scalable graphics",
        "Work with vector documents instead of fixed-resolution painting."
      ]
    ]
  },
  {
    "id": "pixafilm",
    "name": "PixaFilm",
    "category": "Video",
    "use": "video editing",
    "summary": "Turn your footage into a story.",
    "description": "A timeline-based video editor for arranging clips and shaping a sequence.",
    "capabilities": [
      [
        "Timeline editing",
        "Arrange footage along a sequence."
      ],
      [
        "Video and sound",
        "Bring picture and audio together in an editing workspace."
      ],
      [
        "Native projects",
        "Keep footage and editing decisions together in a project document."
      ]
    ]
  },
  {
    "id": "pixalight",
    "name": "PixaLight",
    "category": "Photography",
    "use": "photo development",
    "summary": "Bring out the image you saw.",
    "description": "A photo library and RAW development workspace for organizing photographs and adjusting their look.",
    "capabilities": [
      [
        "Photo library",
        "Organize and navigate a collection of photographs."
      ],
      [
        "RAW development",
        "Adjust image tone and color in a photography-focused workflow."
      ],
      [
        "Referenced originals",
        "Library recovery references your original photos; it does not make backup copies of them."
      ]
    ]
  },
  {
    "id": "pixapdf",
    "name": "PixaPDF",
    "category": "Documents",
    "use": "PDF work",
    "summary": "Bring order to your documents.",
    "description": "A PDF workspace for editing and organizing documents in one focused place.",
    "capabilities": [
      [
        "PDF documents",
        "Open and work with PDF files in a dedicated editor."
      ],
      [
        "Page organization",
        "Arrange documents around the pages you need."
      ],
      [
        "Local workflow",
        "Work on documents saved on your own computer."
      ]
    ]
  },
  {
    "id": "pixadesign",
    "name": "PixaDesign",
    "category": "Publishing",
    "use": "page layout",
    "summary": "Make every page feel considered.",
    "description": "A page-layout workspace for combining typography, artwork and structure in polished documents.",
    "capabilities": [
      [
        "Page composition",
        "Arrange content within page-based layouts."
      ],
      [
        "Design tools",
        "Explore the documented 37-tool toolbar guide."
      ],
      [
        "Product help",
        "Find a starter manual and quick orientation in the editor help."
      ]
    ]
  },
  {
    "id": "pixainterface",
    "name": "PixaInterface",
    "category": "Design",
    "use": "interface design",
    "summary": "From a rough idea to a clear interface.",
    "description": "An interface design and prototyping workspace with native project documents.",
    "capabilities": [
      [
        "Interface layouts",
        "Compose screens and reusable visual elements."
      ],
      [
        "Prototyping",
        "Explore interface ideas within a dedicated design workspace."
      ],
      [
        "Project history",
        "Keep branches and version histories within native project documents."
      ]
    ]
  },
  {
    "id": "pixatexture",
    "name": "PixaTexture",
    "category": "3D",
    "use": "texture painting",
    "summary": "Give your surfaces character.",
    "description": "A 3D texture-painting workspace for creating the surfaces your models need.",
    "capabilities": [
      [
        "Texture painting",
        "Paint surface detail in a dedicated texture workspace."
      ],
      [
        "Material surfaces",
        "Develop the visual character of 3D surfaces."
      ],
      [
        "Native documents",
        "Keep texture work in native project documents."
      ]
    ]
  },
  {
    "id": "pixamaterial",
    "name": "PixaMaterial",
    "category": "3D",
    "use": "material design",
    "summary": "Build a surface from the inside out.",
    "description": "A procedural material workspace for developing and revisiting material designs.",
    "capabilities": [
      [
        "Procedural materials",
        "Explore repeatable material-design workflows."
      ],
      [
        "Surface development",
        "Shape the appearance of materials for your 3D work."
      ],
      [
        "Native projects",
        "Save material work in a dedicated native document."
      ]
    ]
  },
  {
    "id": "pixaword",
    "name": "PixaWord",
    "category": "Documents",
    "use": "writing",
    "summary": "A focused place for the words that matter.",
    "description": "A word-processing workspace for writing, editing and structuring documents.",
    "capabilities": [
      [
        "Writing",
        "Draft and revise text in a document-focused workspace."
      ],
      [
        "Document structure",
        "Shape written work into readable documents."
      ],
      [
        "Draft and revise",
        "Keep your written work editable as it develops."
      ]
    ]
  },
  {
    "id": "pixasound",
    "name": "PixaSound",
    "category": "Audio",
    "use": "audio work",
    "summary": "Give your ideas a voice.",
    "description": "An audio recording and mixing workspace for capturing sound and shaping an audio project.",
    "capabilities": [
      [
        "Recording",
        "Capture audio through a recording workflow."
      ],
      [
        "Mixing",
        "Arrange and balance sound within a desktop workspace."
      ],
      [
        "Project sessions",
        "Keep recordings and your audio work together in a native project."
      ]
    ]
  },
  {
    "id": "pixadeck",
    "name": "PixaDeck",
    "category": "Presentations",
    "use": "presenting",
    "summary": "Build a story worth following.",
    "description": "A presentation workspace for creating and arranging slides around your message.",
    "capabilities": [
      [
        "Slide creation",
        "Build presentations one page at a time."
      ],
      [
        "Visual storytelling",
        "Combine content into a clear presentation sequence."
      ],
      [
        "Native documents",
        "Save decks as editable native projects."
      ]
    ]
  },
  {
    "id": "pixacad",
    "name": "PixaCAD",
    "category": "Technical",
    "use": "CAD and drafting",
    "summary": "Make room for precision.",
    "description": "A CAD and drafting workspace for technical drawing and editable design geometry.",
    "capabilities": [
      [
        "Technical drawing",
        "Work with drawing geometry in a focused CAD environment."
      ],
      [
        "DWG workflows",
        "Explore native drawing workflows; verify important files during product testing."
      ],
      [
        "Block editing",
        "Work with reusable drawing blocks alongside your document geometry."
      ]
    ]
  },
  {
    "id": "pixagrid",
    "name": "PixaGrid",
    "category": "Documents",
    "use": "spreadsheets",
    "summary": "Turn numbers into something useful.",
    "description": "A spreadsheet workspace for organizing information, calculations and workbook-based work.",
    "capabilities": [
      [
        "Workbooks",
        "Organize information across spreadsheet documents."
      ],
      [
        "Formulas",
        "Use calculations to connect values in a workbook."
      ],
      [
        "Native workbooks",
        "Keep your data and formulas together in an editable workbook."
      ]
    ]
  },
  {
    "id": "pixaanything",
    "name": "PixaAnything",
    "category": "Creation",
    "use": "software exploration",
    "summary": "A place to try what comes next.",
    "description": "A software exploration and creation workspace. Its Pixa release and service integrations remain in development.",
    "capabilities": [
      [
        "Explore ideas",
        "Work through software ideas in a dedicated creation workspace."
      ],
      [
        "Draft sessions",
        "Keep drafts and attached files together in a session."
      ],
      [
        "Private recovery",
        "Recovery excludes provider key stores and account credentials; background jobs are not resumed."
      ]
    ]
  }
];
