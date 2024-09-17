import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";

export interface ExtactDocumentImagesStage {
  documentPath: string;
  documentClonePath: string;
  imagesDirectory: string;
}

export interface ExtactDocumentImagesStageSuccess
  extends ExtactDocumentImagesStage {
  documentClonePath: string;
}

export interface ExtactDocumentImagesStageError
  extends ExtactDocumentImagesStage {
  errorMessage: string;
}

export interface PagePreprocessStage {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
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
  preprocessPagesStageResult: PagePreprocessStageResult;
}

export interface PagePreprocessStageError extends PagePreprocessStage {
  errorMessage: string;
}

export interface DocumentProcessStage extends PagePreprocessStageSuccess {
  documentPath: string;
  fileName: string;
}

export interface DocumentProcessStageSuccess extends DocumentProcessStage {}

export interface DocumentProcessStageError extends DocumentProcessStage {
  errorMessage: string;
}

export interface FinishedDocumentProcessStage
  extends DocumentProcessStageSuccess {
  fileNameHistory: string[];
}

export type inProcess =
  | PagePreprocessStage
  | DocumentProcessStage
  | PagePreprocessStageSuccess
  | DocumentProcessStageSuccess
  | PagePreprocessStageError
  | DocumentProcessStageError;

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
  inProcessList: inProcess[];
  pageProcessStageSuccessList: PagePreprocessStageSuccess[];
  pageProcessStageErrorList: PagePreprocessStageError[];
  documentProcessStageSuccessList: DocumentProcessStageSuccess[];
  documentProcessStageErrorList: DocumentProcessStageError[];
  finishedDocumentsProcessStage: FinishedDocumentProcessStage[];
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
    inProcessList: [],
    pageProcessStageSuccessList: [],
    pageProcessStageErrorList: [],
    documentProcessStageSuccessList: [],
    documentProcessStageErrorList: [],
    finishedDocumentsProcessStage: [],
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
    this.state.inProcessList = [];
    this.state.pageProcessStageSuccessList = [];
    this.state.pageProcessStageErrorList = [];
    this.state.documentProcessStageSuccessList = [];
    this.state.documentProcessStageErrorList = [];
    this.state.finishedDocumentsProcessStage = [];
  }
}

export const globalSetupState = $state(new GlobalSetupState(""));
