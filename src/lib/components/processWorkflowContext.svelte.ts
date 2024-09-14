import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";

// {
//   "dates": [
//     {
//       "date": "2018-04-25",
//       "description": "Data de emissão da Nota Fiscal de Serviços (NFS-e)."
//     },
//     {
//       "date": "2011",
//       "description": "Ano de referência da AIDF (Autorização para Impressão de Documentos Fiscais)."
//     }
//   ],
//   "type_name": "Nota Fiscal de Serviços Eletrônica",
//   "type_abbr": "NFS-e",
//   "summary": "Prestação de serviços de impressão de 130 certificados de contribuição para a Federação Nacional das APAEs, totalizando R$ 6.800,00.",
//   "suggested_file_name": "2018-04-25-NFS-E-prestacao_de_servicos_de_impressao_de_130_certificados_de_contribuicao_para_a_federacao_nacional_das_apaes_totalizando_6800"
// }

export interface PagePreprocessStage {
  id: string;
  selectedPages: number[];
  imagesDirectory: string;
  status: "pending" | "completed" | "error";
  startTime: number;
}

export interface PagePreprocessStageResult {
  dates: {
    date: string;
    description: string;
  }[];
  typeName: string;
  typeAbbr: string;
  summary: string;
  suggestedFileName: string;
}

export interface PagePreprocessStageSuccess extends PagePreprocessStage {
  endTime: number;
  elapsedTime: number;
  preprocessPagesStageResult: PagePreprocessStageResult;
}

export interface DocumentProcessStage extends PagePreprocessStageSuccess {
  fileName: string;
}

export interface ProcessError extends PagePreprocessStage {
  endTime: number;
  elapsedTime: number;
  errorMessage: string;
}

export interface FinishedDocumentStage extends DocumentProcessStage {
}

// export interface RenderState {
//   path: string | undefined;
//   dataPath: string | undefined;
//   document: PDFDocumentProxy | undefined;
//   page: PDFPageProxy | undefined;
//   scale: number;
//   rotation: number;
//   numPages: number;
//   pageNumber: number;
//   pageRendering: boolean;
//   pageNumPending: number | undefined;
//   metadata: any | undefined;
//   isActive: boolean;
//   confirmProcessDialogOpen: boolean;
//   showStatusCanvas: boolean;
//   isExtractingImages: boolean;
//   extractedPages: number[];
// }

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
  showStatusCanvas: boolean;
  isExtractingImages: boolean;
  extractedPages: number[];
  selectedPages: number[];
  preprocessPagesStage: PagePreprocessStage[];
  processDocumentsStage: DocumentProcessStage[];
  finishedDocumentsStage: FinishedDocumentStage[];
  processError: ProcessError[];
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
    showStatusCanvas: true,
    isExtractingImages: false,
    extractedPages: [],
    selectedPages: [],
    preprocessPagesStage: [],
    processDocumentsStage: [],
    finishedDocumentsStage: [],
    processError: [],
  });

  constructor(documentPath: string) {
    this.state.documentPath = documentPath;
  }

  get dataPath() {
    const documentPath = this.state.documentPath;
    const dataPath = documentPath.endsWith(".pdf")
      ? documentPath.replace(".pdf", "-data")
      : documentPath + "-data";
    return dataPath;
  }

  get imagesDirectory() {
    const dataPath = this.dataPath;
    const imagesDirectory = `${dataPath}\\images`;
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
    this.state.showStatusCanvas = true;
    this.state.isExtractingImages = false;
    this.state.extractedPages = [];
    this.state.selectedPages = [];
    this.state.preprocessPagesStage = [];
    this.state.processDocumentsStage = [];
    this.state.finishedDocumentsStage = [];
    this.state.processError = [];
  }
}

export const globalSetupState = $state.raw(new GlobalSetupState(""));