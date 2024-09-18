import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";

export interface ExtractDocumentImagesStage {
  documentPath: string;
  imagesDirectory: string;
  dataDirectory: string;
}

export interface PagePreprocessStage {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
}

export class PagePreprocessStageModel implements PagePreprocessStage {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
  }
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

export class PagePreprocessStageResultModel
  implements PagePreprocessStageResult {
  dates: { date: string; description: string }[];
  type_name: string;
  type_abbr: string;
  summary: string;
  suggested_file_name: string;
  constructor(
    dates: { date: string; description: string }[],
    typeName: string,
    typeAbbr: string,
    summary: string,
    suggested_file_name: string,
  ) {
    this.dates = dates;
    this.type_name = typeName;
    this.type_abbr = typeAbbr;
    this.summary = summary;
    this.suggested_file_name = suggested_file_name;
  }
}

export interface PagePreprocessStageSuccess extends PagePreprocessStage {
  pagePreprocessStageResult: PagePreprocessStageResult;
}

export class PagePreprocessStageSuccessModel
  implements PagePreprocessStageSuccess {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  pagePreprocessStageResult: PagePreprocessStageResultModel;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
  }
}

export interface PagePreprocessStageError extends PagePreprocessStage {
  errorMessage: string;
}

export class PagePreprocessStageErrorModel implements PagePreprocessStageError {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  errorMessage: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    errorMessage: string,
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.errorMessage = errorMessage;
  }
}

export interface DocumentProcessStage extends PagePreprocessStageSuccess {
  documentPath: string;
  fileName: string;
}

export class DocumentProcessStageModel implements DocumentProcessStage {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  pagePreprocessStageResult: PagePreprocessStageResultModel;
  documentPath: string;
  fileName: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.documentPath = documentPath;
    this.fileName = fileName;
  }
}

export interface DocumentProcessStageSuccess extends DocumentProcessStage {
  pagePreprocessStageResult: PagePreprocessStageResult;
}

export class DocumentProcessStageSuccessModel
  implements DocumentProcessStageSuccess {
  stage = $state<DocumentProcessStageSuccess>({
    id: "",
    selectedPages: [],
    dataDirectory: "",
    imagesDirectory: "",
    pagePreprocessStageResult: new PagePreprocessStageResultModel([], "", "", "", ""),
    documentPath: "",
    fileName: "",
  });
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
  ) {
    this.stage.id = id;
    this.stage.selectedPages = selectedPages;
    this.stage.dataDirectory = dataDirectory;
    this.stage.imagesDirectory = imagesDirectory;
    this.stage.pagePreprocessStageResult = pagePreprocessStageResult;
    this.stage.documentPath = documentPath;
    this.stage.fileName = fileName;
  }
  get id() {
    return this.stage.id;
  }
  get selectedPages() {
    return this.stage.selectedPages;
  }
  get dataDirectory() {
    return this.stage.dataDirectory;
  }
  get imagesDirectory() {
    return this.stage.imagesDirectory;
  }
  get pagePreprocessStageResult() {
    return this.stage.pagePreprocessStageResult;
  }
  get documentPath() {
    return this.stage.documentPath;
  }
  get fileName() {
    return this.stage.fileName;
  }
}

export interface DocumentProcessStageError extends DocumentProcessStage {
  errorMessage: string;
}

export class DocumentProcessStageErrorModel
  implements DocumentProcessStageError {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  pagePreprocessStageResult: PagePreprocessStageResult;
  documentPath: string;
  fileName: string;
  errorMessage: string;

  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
    errorMessage: string,
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.documentPath = documentPath;
    this.fileName = fileName;
    this.errorMessage = errorMessage;
  }
}

export interface FinishedDocumentProcessStage
  extends DocumentProcessStageSuccess {
  fileNameHistory: string[];
}

export class FinishedDocumentProcessStageModel
  implements FinishedDocumentProcessStage {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  pagePreprocessStageResult: PagePreprocessStageResult;
  documentPath: string;
  fileName: string;
  fileNameHistory: string[];

  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
    fileNameHistory: string[],
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.documentPath = documentPath;
    this.fileName = fileName;
    this.fileNameHistory = fileNameHistory;
  }
}

export type Stage =
  | PagePreprocessStageModel
  | PagePreprocessStageErrorModel
  | DocumentProcessStageModel
  | DocumentProcessStageErrorModel;

export class InProcessInstanceModel {
  stage: Stage;
  id: string;
  constructor(stage: Stage) {
    this.stage = stage;
    const id = stage.id;
    this.id = id;
  }
}

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
  inProcessList: InProcessInstanceModel[];
  pageProcessStageSuccessList: PagePreprocessStageSuccessModel[];
  pageProcessStageErrorList: PagePreprocessStageErrorModel[];
  documentProcessStageSuccessList: DocumentProcessStageSuccessModel[];
  documentProcessStageErrorList: DocumentProcessStageErrorModel[];
  finishedDocumentsProcessStage: FinishedDocumentProcessStageModel[];
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
