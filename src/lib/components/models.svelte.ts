import { invoke } from "@tauri-apps/api/core";
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
  pageNumberPrefix: string;
}

export class PagePreprocessStageSuccessModel
  implements PagePreprocessStageSuccess {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  pagePreprocessStageResult: PagePreprocessStageResultModel;
  pageNumberPrefix: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    pageNumberPrefix: string,
  ) {
    this.id = id;

    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.pageNumberPrefix = pageNumberPrefix;
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
  pageNumberPrefix: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
    pageNumberPrefix: string,
  ) {
    this.id = id;

    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.documentPath = documentPath;
    this.fileName = fileName;
    this.pageNumberPrefix = pageNumberPrefix;
  }
}

export interface DocumentProcessStageSuccess extends DocumentProcessStage {
  pagePreprocessStageResult: PagePreprocessStageResult;
}

export class DocumentProcessStageSuccessModel
  implements DocumentProcessStageSuccess {
  id: string;
  selectedPages: number[];
  dataDirectory: string;
  imagesDirectory: string;
  pagePreprocessStageResult: PagePreprocessStageResultModel;
  documentPath: string;
  fileName: string;
  pageNumberPrefix: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
    pageNumberPrefix: string,
  ) {
    this.id = id;

    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = new PagePreprocessStageResultModel(
      pagePreprocessStageResult.dates,
      pagePreprocessStageResult.type_name,
      pagePreprocessStageResult.type_abbr,
      pagePreprocessStageResult.summary,
      pagePreprocessStageResult.suggested_file_name,
    );
    this.documentPath = documentPath;
    this.fileName = fileName;
    this.pageNumberPrefix = pageNumberPrefix;
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
  pageNumberPrefix: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
    errorMessage: string,
    pageNumberPrefix: string,
  ) {
    this.id = id;

    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.documentPath = documentPath;
    this.fileName = fileName;
    this.errorMessage = errorMessage;
    this.pageNumberPrefix = pageNumberPrefix;
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
  pageNumberPrefix: string;
  constructor(
    id: string,
    selectedPages: number[],
    dataDirectory: string,
    imagesDirectory: string,
    pagePreprocessStageResult: PagePreprocessStageResult,
    documentPath: string,
    fileName: string,
    fileNameHistory: string[],
    pageNumberPrefix: string,
  ) {
    this.id = id;
    this.selectedPages = selectedPages;
    this.dataDirectory = dataDirectory;
    this.imagesDirectory = imagesDirectory;
    this.pagePreprocessStageResult = pagePreprocessStageResult;
    this.documentPath = documentPath;
    this.fileName = fileName;
    this.fileNameHistory = fileNameHistory;
    this.pageNumberPrefix = pageNumberPrefix;
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

  async updateFileName({ id, newFileName }: { id: string, newFileName: string }) {
    const index = this.state.finishedDocumentsProcessStage.findIndex((doc) => doc.id === id);
    if (index === -1) {
      console.error(`Document with id ${id} not found`);
      return;
    }
    const document = this.state.finishedDocumentsProcessStage[index];
    this.state.finishedDocumentsProcessStage.splice(index, 1);
    const originalFileName = document.fileName;
    try {

      const cleanNewFileName = newFileName.replace(document.pageNumberPrefix+"-", "");
      const newDocumentPath = await invoke<string>("run_update_file_name", {
        newFileName: cleanNewFileName,
        documentPath: document.documentPath,
      });
      const fileNameHistory = document.fileNameHistory.includes(cleanNewFileName)
        ? document.fileNameHistory
        : [...document.fileNameHistory, cleanNewFileName];
      const newFinishedDocumentStage = new FinishedDocumentProcessStageModel(
        document.id,
        document.selectedPages,
        document.dataDirectory,
        document.imagesDirectory,
        document.pagePreprocessStageResult,
        newDocumentPath,
        cleanNewFileName,
        fileNameHistory,
        document.pageNumberPrefix,
      );
      this.state.finishedDocumentsProcessStage.push(newFinishedDocumentStage);
      console.log(`File name updated successfully: ${originalFileName} -> ${cleanNewFileName}`);
      return cleanNewFileName;
    } catch (error) {
      console.error("Error updating file name:", error);
      this.state.finishedDocumentsProcessStage.splice(index, 0, document);
    }
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
