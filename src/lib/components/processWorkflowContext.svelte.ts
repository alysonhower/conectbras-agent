import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";

export interface ExtactDocumentImagesStage {
  documentPath: string;
  documentClonePath: string;
  imagesDirectory: string;
  startTime: number;
}

export interface ExtactDocumentImagesStageSuccess
  extends ExtactDocumentImagesStage {
  endTime: number;
  elapsedTime: number;
  documentClonePath: string;
}

export interface ExtactDocumentImagesStageError
  extends ExtactDocumentImagesStage {
  endTime: number;
  elapsedTime: number;
  errorMessage: string;
}

export interface PagePreprocessStage {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  startTime: number;
}

export interface PagePreprocessStageResult {
  dates: {
    date: string;
    description: string;
  }[];
  type_name: string;
  type_abbr: string;
  summary: string;
  suggested_file_name: string;
}

export interface PagePreprocessStageSuccess extends PagePreprocessStage {
  endTime: number;
  elapsedTime: number;
  preprocessPagesStageResult: PagePreprocessStageResult;
}

export interface DocumentProcessStage extends PagePreprocessStageSuccess {
  documentPath: string;
  fileName: string;
  fileNameHistory: string[];
}

export interface PagePreprocessStageError extends PagePreprocessStage {
  endTime: number;
  elapsedTime: number;
  errorMessage: string;
}

export interface FinishedDocumentProcessStage extends DocumentProcessStage {}

interface SetupState {
  documentPath: string;
  documentProxy: PDFDocumentProxy | undefined;
  pageProxy: PDFPageProxy | undefined;
  scale: number;
  rotation: number;
  numPages: number;
  pageNumber: number;
  pageRendering: boolean;
  pageNumPending: number | undefined;
  metadata: any | undefined;
  isActive: boolean;
  isDialogOpen: boolean;
  isShowStatusCanvas: boolean;
  isShowShortcuts: boolean;
  isExtractingImages: boolean;
  extractedPages: number[];
  selectedPages: number[];
  pagesProcessStage: PagePreprocessStage[];
  documentsProcessStage: DocumentProcessStage[];
  finishedDocumentsProcessStage: FinishedDocumentProcessStage[];
  processErrors: PagePreprocessStageError[];
}

class GlobalSetupState {
  state = $state<SetupState>({
    documentPath: "",
    documentProxy: undefined,
    pageProxy: undefined,
    scale: 1,
    rotation: 0,
    numPages: 0,
    pageNumber: 1,
    pageRendering: false,
    pageNumPending: undefined,
    metadata: undefined,
    isActive: false,
    isDialogOpen: false,
    isShowStatusCanvas: true,
    isExtractingImages: false,
    isShowShortcuts: false,
    extractedPages: [],
    selectedPages: [],
    pagesProcessStage: [],
    documentsProcessStage: [],
    finishedDocumentsProcessStage: [],
    processErrors: [],
  });

  constructor(documentPath: string) {
    this.state.documentPath = documentPath;
  }

  get documentClonePath() {
    const documentPath = this.state.documentPath;
    const dataDirectory = documentPath.endsWith(".pdf")
      ? documentPath.replace(".pdf", "-data")
      : documentPath + "-data";
    const documentClonePath = `${dataDirectory}\\${documentPath.split("\\").pop()}`;
    return documentClonePath;
  }

  get dataDirectory() {
    const documentPath = this.state.documentPath;
    const dataDirectory = documentPath.endsWith(".pdf")
      ? documentPath.replace(".pdf", "-data")
      : documentPath + "-data";
    return dataDirectory;
  }

  get imagesDirectory() {
    const dataDirectory = this.dataDirectory;
    const imagesDirectory = `${dataDirectory}\\images`;
    return imagesDirectory;
  }

  async clearState() {
    await this.state.documentProxy?.destroy();
    this.state.documentPath = "";
    this.state.documentProxy = undefined;
    this.state.pageProxy = undefined;
    this.state.scale = 1;
    this.state.rotation = 0;
    this.state.numPages = 0;
    this.state.pageNumber = 1;
    this.state.pageRendering = false;
    this.state.pageNumPending = undefined;
    this.state.metadata = undefined;
    this.state.isActive = false;
    this.state.isDialogOpen = false;
    this.state.isShowStatusCanvas = true;
    this.state.isExtractingImages = false;
    this.state.extractedPages = [];
    this.state.selectedPages = [];
    this.state.pagesProcessStage = [];
    this.state.documentsProcessStage = [];
    this.state.finishedDocumentsProcessStage = [];
    this.state.processErrors = [];
  }
}

export const globalSetupState = $state(new GlobalSetupState(""));
