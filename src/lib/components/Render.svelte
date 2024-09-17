<script lang="ts">
  import * as pdfjs from "pdfjs-dist";
  import { tick, untrack } from "svelte";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Dialog from "$lib/components/ui/dialog";
  import {
    ZoomIn,
    ZoomOut,
    ChevronFirst,
    ArrowLeft,
    ArrowRight,
    ChevronLast,
    RotateCcw,
    RotateCw,
    FolderOpen,
    FilePlus,
    FileMinus,
    FileCheck,
    Eye,
    EyeOff,
    Loader2,
    Keyboard,
  } from "lucide-svelte/icons";
  import { homeDir, resolve } from "@tauri-apps/api/path";
  import {
    readFile,
    exists,
    mkdir,
    readDir,
    remove,
  } from "@tauri-apps/plugin-fs";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { TextContent, TextItem } from "pdfjs-dist/types/src/display/api";
  import { invoke } from "@tauri-apps/api/core";
  import { v4 as uuidv4 } from "uuid";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import { ChevronDown, ChevronUp } from "lucide-svelte/icons";
  import { emit, listen } from "@tauri-apps/api/event";
  import {
    globalSetupState,
    // ExtractDocumentImagesStageModel,
    PagePreprocessStageSuccessModel,
    PagePreprocessStageModel,
    DocumentProcessStageModel,
    FinishedDocumentProcessStageModel,
    DocumentProcessStageSuccessModel,
    PagePreprocessStageResultModel,
    PagePreprocessStageErrorModel,
    DocumentProcessStageErrorModel,
  } from "./processWorkflowContext.svelte";

  interface ProgressUpdate {
    pages_processed: number;
    pages_to_process: number;
    total_document_pages: number;
    estimated_seconds_remaining: number;
    extracted_page_numbers: number[];
  }

  const renderState = globalSetupState.state;

  const MIN_SCALE = 0.4;
  const MAX_SCALE = 10;
  const ROTATION_INCREMENT = 90;
  const ZOOM_INCREMENT = 0.1;

  pdfjs.GlobalWorkerOptions.workerSrc = new URL(
    "pdfjs-dist/build/pdf.worker.mjs",
    import.meta.url,
  ).toString();

  let component: HTMLDivElement;
  let canvasContainer: HTMLDivElement;
  let renderCanvas: HTMLCanvasElement;
  let statusCanvas: HTMLCanvasElement;
  let textLayer: SVGSVGElement;

  let isWorkflowExpanded = $state(false);

  const toggleWorkflow = () => {
    isWorkflowExpanded = !isWorkflowExpanded;
  };

  const loadDocument = async () => {
    try {
      const data = await readFile(globalSetupState.state.documentPath!);
      const documentProxy = await pdfjs.getDocument({ data }).promise;
      renderState.documentProxy = documentProxy;
      renderState.numPages = documentProxy.numPages;
      const metadata = await documentProxy.getMetadata();
      renderState.metadata = metadata;
      console.log("Document loaded!\nMetadata:\n", metadata);
    } catch (error) {
      handleError("Error loading document:", error);
    }
  };

  const buildPDFTextLayer = (
    viewport: pdfjs.PageViewport,
    textContent: TextContent,
  ) => {
    const svg = textLayer;
    svg.innerHTML = "";
    svg.setAttribute("width", `${viewport.width}px`);
    svg.setAttribute("height", `${viewport.height}px`);
    svg.setAttribute("font-size", "1");
    textContent.items.forEach((item) => {
      if ("str" in item) {
        const textItem = item as TextItem;
        const tx = pdfjs.Util.transform(
          pdfjs.Util.transform(viewport.transform, textItem.transform),
          [1, 0, 0, -1, 0, 0],
        );
        const style = textContent.styles[textItem.fontName];
        const text = document.createElementNS(
          "http://www.w3.org/2000/svg",
          "svg:text",
        );
        text.setAttribute("transform", `matrix(${tx.join(" ")})`);
        text.setAttribute("font-family", style.fontFamily);
        text.setAttribute("fill", "transparent");
        text.textContent = textItem.str;
        svg.append(text);
      }
    });
  };

  const applyStatusCanvasStyles = (
    pageNumber: number,
    canvasContext: CanvasRenderingContext2D,
    viewportWidth: number,
    viewportHeight: number,
  ) => {
    canvasContext.clearRect(0, 0, viewportWidth, viewportHeight);

    let text: string;
    let bgColor: string;
    let textColor: string;
    let borderColor: string;

    if (
      renderState.finishedDocumentsProcessStage.some((fd) =>
        fd?.selectedPages?.includes(pageNumber),
      )
    ) {
      text = `Página ${pageNumber}: Processamento concluído`;
      bgColor = "rgba(128, 0, 128, 0.2)";
      textColor = "rgba(76, 0, 76, 1)";
      borderColor = "rgba(128, 0, 128, 1)";
    } else if (
      renderState.documentProcessStageSuccessList.some((pp) =>
        pp?.selectedPages?.includes(pageNumber),
      )
    ) {
      text = `Página ${pageNumber}: Documento gerado`;
      bgColor = "rgba(0, 128, 0, 0.2)";
      textColor = "rgba(0, 76, 0, 1)";
      borderColor = "rgba(0, 128, 0, 1)";
    } else if (
      renderState.pageProcessStageSuccessList.some((pp) =>
        pp?.selectedPages?.includes(pageNumber),
      )
    ) {
      text = `Página ${pageNumber}: Pré-processada`;
      bgColor = "rgba(144, 238, 144, 0.2)";
      textColor = "rgba(0, 100, 0, 1)";
      borderColor = "rgba(144, 238, 144, 1)";
    } else if (
      renderState.documentProcessStageErrorList.some((pp) =>
        pp.selectedPages.includes(pageNumber),
      )
    ) {
      text = `Página ${pageNumber}: Erro na geração do documento`;
      bgColor = "rgba(220, 20, 60, 0.2)";
      textColor = "rgba(139, 0, 0, 1)";
      borderColor = "rgba(220, 20, 60, 1)";
    } else if (
      renderState.pageProcessStageErrorList.some((pp) =>
        pp.selectedPages.includes(pageNumber),
      )
    ) {
      text = `Página ${pageNumber}: Erro no pré-processamento`;
      bgColor = "rgba(255, 99, 71, 0.2)";
      textColor = "rgba(178, 34, 34, 1)";
      borderColor = "rgba(255, 99, 71, 1)";
    } else if (
      renderState.inProcessList.some((pp) =>
        pp.selectedPages.includes(pageNumber),
      )
    ) {
      const process = renderState.inProcessList.find((pp) =>
        pp.selectedPages.includes(pageNumber),
      );
      if (process instanceof PagePreprocessStageModel) {
        text = `Página ${pageNumber}: Pré-processando...`;
        bgColor = "rgba(255, 255, 224, 0.2)";
        textColor = "rgba(184, 134, 11, 1)";
        borderColor = "rgba(255, 255, 224, 1)";
      } else if (process instanceof DocumentProcessStageModel) {
        text = `Página ${pageNumber}: Gerando documento...`;
        bgColor = "rgba(255, 165, 0, 0.2)";
        textColor = "rgba(210, 105, 30, 1)";
        borderColor = "rgba(255, 165, 0, 1)";
      } else if (process instanceof PagePreprocessStageSuccessModel) {
        text = `Página ${pageNumber}: Pré-processamento concluído`;
        bgColor = "rgba(152, 251, 152, 0.2)";
        textColor = "rgba(0, 128, 0, 1)";
        borderColor = "rgba(152, 251, 152, 1)";
      } else if (process instanceof DocumentProcessStageSuccessModel) {
        text = `Página ${pageNumber}: Documento gerado`;
        bgColor = "rgba(0, 255, 127, 0.2)";
        textColor = "rgba(0, 100, 0, 1)";
        borderColor = "rgba(0, 255, 127, 1)";
      } else if (process instanceof PagePreprocessStageErrorModel) {
        text = `Página ${pageNumber}: Falha no pré-processamento`;
        bgColor = "rgba(255, 182, 193, 0.2)";
        textColor = "rgba(178, 34, 34, 1)";
        borderColor = "rgba(255, 182, 193, 1)";
      } else if (process instanceof DocumentProcessStageErrorModel) {
        text = `Página ${pageNumber}: Falha na geração do documento`;
        bgColor = "rgba(255, 99, 71, 0.2)";
        textColor = "rgba(139, 0, 0, 1)";
        borderColor = "rgba(255, 99, 71, 1)";
      } else {
        text = `Página ${pageNumber}: Processando...`;
        bgColor = "rgba(135, 206, 250, 0.2)";
        textColor = "rgba(30, 144, 255, 1)";
        borderColor = "rgba(135, 206, 250, 1)";
      }
    } else if (renderState.selectedPages.includes(pageNumber)) {
      text = `Página ${pageNumber}: Selecionada`;
      bgColor = "rgba(173, 216, 230, 0.5)";
      textColor = "rgba(0, 90, 156, 1)";
      borderColor = "rgba(70, 130, 180, 0.8)";
    } else {
      return;
    }

    let fontSize = Math.min(viewportWidth, viewportHeight) * 0.07;
    canvasContext.font = `bold ${fontSize}px Arial`;
    let textWidth = canvasContext.measureText(text).width;
    while (textWidth > viewportWidth * 0.9 && fontSize > 10) {
      fontSize -= 1;
      canvasContext.font = `bold ${fontSize}px Arial`;
      textWidth = canvasContext.measureText(text).width;
    }

    const x = (viewportWidth - textWidth) / 2;
    const y = (viewportHeight + fontSize) / 2 - fontSize / 2;

    canvasContext.strokeStyle = borderColor;
    canvasContext.lineWidth = 8;
    canvasContext.strokeRect(4, 4, viewportWidth - 8, viewportHeight - 8);

    canvasContext.fillStyle = bgColor;
    canvasContext.fillRect(8, 8, viewportWidth - 16, viewportHeight - 16);

    canvasContext.shadowColor = "rgba(255, 255, 255, 0.7)";
    canvasContext.shadowBlur = 5;
    canvasContext.shadowOffsetX = 1;
    canvasContext.shadowOffsetY = 1;
    canvasContext.fillStyle = textColor;
    canvasContext.fillText(text, x, y);

    canvasContext.strokeStyle = "rgba(0, 0, 0, 0.4)";
    canvasContext.lineWidth = 3;
    canvasContext.strokeText(text, x, y);
  };

  const loadPage = async (pageNumber: number) => {
    renderState.pageRendering = true;

    try {
      const page = await renderState.documentProxy!.getPage(pageNumber);
      const textContent = await page.getTextContent();

      const viewport = page.getViewport({
        scale: renderState.scale,
        rotation: renderState.rotation,
      });

      const { height, width } = viewport;

      renderCanvas.height = height;
      renderCanvas.width = width;
      statusCanvas.height = height;
      statusCanvas.width = width;

      const canvasContext = renderCanvas.getContext("2d");
      const statusCanvasContext = statusCanvas.getContext("2d");

      if (canvasContext && statusCanvasContext) {
        applyStatusCanvasStyles(pageNumber, statusCanvasContext, width, height);
        await tick();

        const renderContext = {
          canvasContext,
          viewport,
        };

        await page.render(renderContext).promise;

        buildPDFTextLayer(viewport, textContent);
      }

      renderState.pageRendering = false;

      if (renderState.pageNumPending !== undefined) {
        await tick();
        loadPageQueue(renderState.pageNumPending);
        renderState.pageNumPending = undefined;
      }
    } catch (error) {
      handleError("Error loading page:", error);
      renderState.pageRendering = false;
    }
  };

  const loadPageQueue = (pageNumber: number) => {
    if (renderState.pageRendering) {
      renderState.pageNumPending = pageNumber;
    } else {
      loadPage(pageNumber);
    }
  };

  const updatePageNumber = (delta: number) => {
    renderState.pageNumber = Math.max(
      1,
      Math.min(
        renderState.documentProxy!.numPages,
        renderState.pageNumber + delta,
      ),
    );
  };

  const updateScale = (delta: number) => {
    const scaleFactor = Math.exp(delta);
    renderState.scale = Math.min(
      MAX_SCALE,
      Math.max(MIN_SCALE, renderState.scale * scaleFactor),
    );
  };

  const updateRotation = (delta: number) => {
    renderState.rotation = (renderState.rotation + delta + 360) % 360;
  };

  const handleFirstPage = () => (renderState.pageNumber = 1);
  const handleLastPage = () => (renderState.pageNumber = renderState.numPages);
  const handlePrevPage = () => updatePageNumber(-1);
  const handleNextPage = () => updatePageNumber(1);
  const handleZoomIn = () => updateScale(ZOOM_INCREMENT);
  const handleZoomOut = () => updateScale(-ZOOM_INCREMENT);
  const handleRotateLeft = () => updateRotation(-ROTATION_INCREMENT);
  const handleRotateRight = () => updateRotation(ROTATION_INCREMENT);

  const handleSelectPDF = async () => {
    try {
      const file = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
        title: "Por favor, selecione um PDF",
        defaultPath: await homeDir(),
      });
      if (file) globalSetupState.state.documentPath = file;
    } catch (error) {
      handleError("Error selecting PDF:", error);
    }
  };

  const handleSelectPage = () => {
    if (!renderState.extractedPages.includes(validPageNumber)) return;
    if (
      renderState.pageProcessStageSuccessList.some((pp) =>
        pp.selectedPages.includes(validPageNumber),
      ) ||
      renderState.documentProcessStageSuccessList.some((pp) =>
        pp.selectedPages.includes(validPageNumber),
      ) ||
      renderState.finishedDocumentsProcessStage.some((pp) =>
        pp.selectedPages.includes(validPageNumber),
      )
    )
      return;
    const pageNumber = validPageNumber;
    const index = renderState.selectedPages.indexOf(pageNumber);

    if (index !== -1) {
      renderState.selectedPages.splice(index, 1);
    } else {
      renderState.selectedPages.push(pageNumber);
    }
    renderState.selectedPages.sort((a, b) => a - b);
    updatePageNumberAfterSelection(pageNumber);
    console.log(
      renderState.selectedPages.length > 0
        ? selectedPagesText
        : "Nenhuma página selecionada.",
    );
  };

  const updatePageNumberAfterSelection = (pageNumber: number) => {
    const currentIndex = renderState.selectedPages.indexOf(pageNumber);
    const availablePages = renderState.extractedPages.filter(
      (page) =>
        !renderState.inProcessList.some((pp) =>
          pp.selectedPages.includes(page),
        ) &&
        !renderState.pageProcessStageSuccessList.some((pp) =>
          pp.selectedPages.includes(page),
        ) &&
        !renderState.documentProcessStageSuccessList.some((pp) =>
          pp.selectedPages.includes(page),
        ) &&
        !renderState.finishedDocumentsProcessStage.some((pp) =>
          pp.selectedPages.includes(page),
        ),
    );

    if (currentIndex === -1) {
      const prevSelectedPage = renderState.selectedPages
        .slice()
        .reverse()
        .find((page) => availablePages.includes(page) && page < pageNumber);
      const nextSelectedPage = renderState.selectedPages.find(
        (page) => availablePages.includes(page) && page > pageNumber,
      );
      renderState.pageNumber =
        prevSelectedPage || nextSelectedPage || pageNumber;
    } else {
      const prevUnselectedPage = availablePages
        .slice()
        .reverse()
        .find(
          (page) =>
            page < pageNumber && !renderState.selectedPages.includes(page),
        );
      const nextUnselectedPage = availablePages.find(
        (page) =>
          page > pageNumber && !renderState.selectedPages.includes(page),
      );
      renderState.pageNumber =
        prevUnselectedPage || nextUnselectedPage || pageNumber;
    }
  };

  const handlePageNumberClick = (pageNumber: number) => {
    renderState.pageNumber === pageNumber
      ? handleSelectPage()
      : (renderState.pageNumber = pageNumber);
  };

  const handleMouseDown = (e: MouseEvent) => {
    if (e.detail > 1) e.preventDefault();
    const target = e.target as Node;
    renderState.isActive =
      component.contains(target) || canvasContainer.contains(target);
  };

  const handleDoubleClick = (e: MouseEvent) => {
    if (!renderState.numPages || !canvasContainer.contains(e.target as Node))
      return;
    const target = e.target as HTMLElement;
    if (target.closest(".toggle-status-button")) return;
    handleSelectPage();
  };

  const handleWheel = (e: WheelEvent) => {
    if (!renderState.numPages) return;

    renderState.isActive = component.contains(e.target as Node);

    if (!renderState.isActive) return;

    if (e.ctrlKey && e.shiftKey) {
      updateRotation(e.deltaY < 0 ? ROTATION_INCREMENT : -ROTATION_INCREMENT);
    } else if (e.ctrlKey) {
      updateScale(e.deltaY < 0 ? ZOOM_INCREMENT : -ZOOM_INCREMENT);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      if (renderState.isDialogOpen) {
        renderState.isDialogOpen = false;
      }
      return;
    }

    if (e.ctrlKey && (e.key === "o" || e.key === "O")) {
      e.preventDefault();
      handleSelectPDF();
      return;
    }
    if (!renderState.isActive || !renderState.numPages) return;

    switch (e.key) {
      case "Home":
        handleFirstPage();
        break;
      case "End":
        handleLastPage();
        break;
      case "ArrowLeft":
        e.shiftKey ? handleFirstPage() : handlePrevPage();
        break;
      case "ArrowRight":
        e.shiftKey ? handleLastPage() : handleNextPage();
        break;
      case " ":
        e.preventDefault();
        handleSelectPage();
        break;
      case "Backspace":
        if (renderState.selectedPages.length)
          renderState.pageNumber = renderState.selectedPages.pop()!;
        break;
      case "Enter":
        if (renderState.selectedPages.length && !renderState.isDialogOpen) {
          e.preventDefault();
          renderState.isDialogOpen = true;
        }
        break;
      case "Tab":
        e.preventDefault();
        if (e.shiftKey) {
          handlePrevPage();
        } else {
          handleNextPage();
        }
        break;
    }

    if (e.ctrlKey) {
      switch (e.key) {
        case "=":
          handleZoomIn();
          break;
        case "-":
          handleZoomOut();
          break;
        case "ArrowLeft":
          handleRotateLeft();
          break;
        case "ArrowRight":
          handleRotateRight();
          break;
      }
    }

    if (e.shiftKey) {
      switch (e.key) {
        case "ArrowUp":
          handleZoomIn();
          break;
        case "ArrowDown":
          handleZoomOut();
          break;
      }
    }
  };

  const handleProcessPages = async () => {
    if (!globalSetupState.imagesDirectory) return;
    const selectedPages = $state.snapshot(renderState.selectedPages);
    const dataDirectory = $state.snapshot(globalSetupState.dataDirectory);
    const imagesDirectory = $state.snapshot(globalSetupState.imagesDirectory);
    const pagePreprocessStage = new PagePreprocessStageModel(
      uuidv4(),
      selectedPages,
      dataDirectory,
      imagesDirectory,
    );

    renderState.inProcessList.push(pagePreprocessStage);
    renderState.selectedPages.splice(0, renderState.selectedPages.length);

    try {
      const runPagePreprocessStage =
        await invoke<PagePreprocessStageSuccessModel>(
          "run_page_preprocess_stage",
          {
            pagePreprocessStage,
          },
        );

      const pagePreprocessStageResult = new PagePreprocessStageResultModel(
        runPagePreprocessStage.pagePreprocessStageResult.dates,
        runPagePreprocessStage.pagePreprocessStageResult.type_name,
        runPagePreprocessStage.pagePreprocessStageResult.type_abbr,
        runPagePreprocessStage.pagePreprocessStageResult.summary,
        runPagePreprocessStage.pagePreprocessStageResult.suggested_file_name,
      );

      console.log(
        "new PagePreprocessStageResultModel:",
        pagePreprocessStageResult,
      );

      const pagePreprocessStageSuccess = new PagePreprocessStageSuccessModel(
        runPagePreprocessStage.id,
        runPagePreprocessStage.selectedPages,
        runPagePreprocessStage.imagesDirectory,
        runPagePreprocessStage.dataDirectory,
        pagePreprocessStageResult,
      );

      console.log(
        "new PagePreprocessStageSuccessModel:",
        pagePreprocessStageSuccess,
      );

      const fileName =
        pagePreprocessStageSuccess.pagePreprocessStageResult
          .suggested_file_name;

      const ppsIndex = renderState.inProcessList.findIndex(
        (pps) => pps.id === pagePreprocessStageSuccess.id,
      );

      if (ppsIndex !== -1) {
        renderState.inProcessList.splice(ppsIndex, 1);
      }

      const documentProcessStage = new DocumentProcessStageModel(
        pagePreprocessStageSuccess.id,
        pagePreprocessStageSuccess.selectedPages,
        pagePreprocessStageSuccess.dataDirectory,
        pagePreprocessStageSuccess.imagesDirectory,
        pagePreprocessStageResult,
        globalSetupState.documentClonePath,
        fileName,
      );

      console.log("new DocumentProcessStageModel:", documentProcessStage);

      renderState.inProcessList.push(documentProcessStage);

      try {
        const documentProcessStageSuccess: DocumentProcessStageSuccessModel =
          await invoke("run_document_process_stage", {
            documentProcessStage,
          });

        // const documentProcessStageResult = new PagePreprocessStageResultModel(
        //   result.pagePreprocessStageResult.dates,
        //   result.pagePreprocessStageResult.type_name,
        //   result.pagePreprocessStageResult.type_abbr,
        //   result.pagePreprocessStageResult.summary,
        //   result.pagePreprocessStageResult.suggested_file_name,
        // );

        // console.log(
        //   "new DocumentProcessStageResultModel:",
        //   documentProcessStageSuccess,
        // );

        // const documentProcessStageSuccess =
        //   new DocumentProcessStageSuccessModel(
        //     result.id,
        //     result.selectedPages,
        //     result.dataDirectory,
        //     result.imagesDirectory,
        //     documentProcessStageResult,
        //     result.documentPath,
        //     result.fileName,
        //   );

        console.log(
          "new DocumentProcessStageSuccessModel:",
          documentProcessStageSuccess,
        );

        const dpsIndex = renderState.inProcessList.findIndex(
          (pps) => pps.id === documentProcessStageSuccess.id,
        );

        if (dpsIndex !== -1) {
          renderState.inProcessList.splice(dpsIndex, 1);
        }

        const finishedDocumentProcessStage =
          new FinishedDocumentProcessStageModel(
            documentProcessStageSuccess.id,
            documentProcessStageSuccess.selectedPages,
            documentProcessStageSuccess.dataDirectory,
            documentProcessStageSuccess.imagesDirectory,
            documentProcessStageSuccess.pagePreprocessStageResult,
            documentProcessStageSuccess.documentPath,
            documentProcessStageSuccess.fileName,
            [documentProcessStageSuccess.fileName],
          );

        console.log(
          "new FinishedDocumentProcessStageModel:",
          finishedDocumentProcessStage,
        );

        renderState.finishedDocumentsProcessStage.push(
          finishedDocumentProcessStage,
        );
      } catch (e) {
        console.log("Error in run_document_process_stage raw error:", e);
        const error = e as DocumentProcessStageErrorModel;
        console.log("new DocumentProcessStageErrorModel:", error);
        const dpsIndex = renderState.inProcessList.findIndex(
          (pps) => pps.id === error.id,
        );

        if (dpsIndex !== -1) {
          renderState.inProcessList.splice(dpsIndex, 1);
        }
        renderState.documentProcessStageErrorList.push(error);
      }
    } catch (e) {
      const error = e as PagePreprocessStageErrorModel;

      console.log("new PagePreprocessStageErrorModel:", error);
      const ppsIndex = renderState.inProcessList.findIndex(
        (pps) => pps.id === error.id,
      );
      if (ppsIndex !== -1) {
        renderState.inProcessList.splice(ppsIndex, 1);
      }

      renderState.pageProcessStageErrorList.push(error);
    }
  };

  const selectedPagesText = $derived.by(() => {
    if (renderState.selectedPages.length === 1) {
      return `Página ${renderState.selectedPages[0]} selecionada.`;
    } else if (renderState.selectedPages.length === 2) {
      return `Páginas ${renderState.selectedPages[0]} e ${renderState.selectedPages[1]} selecionadas.`;
    } else {
      return `Páginas selecionadas: ${renderState.selectedPages.slice(0, -1).join(", ")} e ${renderState.selectedPages.slice(-1)}.`;
    }
  });

  const validPageNumber = $derived(
    Math.min(Math.max(1, renderState.pageNumber), renderState.numPages),
  );

  const isPageAvailable = $derived(
    renderState.extractedPages.includes(validPageNumber),
  );

  function handleError(message: string, error: any) {
    console.error(message, error);
    // TODO: Implement user-facing error handling
  }

  // const invokeExtractDocumentImagesStage = async (
  //   extractDocumentImagesStage: ExtractDocumentImagesStageModel,
  // ) => {
  //   console.log("new ExtractDocumentImagesStage:", extractDocumentImagesStage);
  //   try {
  //     const result = await invoke("run_extract_document_images_stage", {
  //       extractDocumentImagesStage,
  //     });
  //     console.log("extract_document_images:", result);
  //   } catch (e) {
  //     console.error("Error in extract_document_images:", e);
  //   }
  // };

  $effect(() => {
    if (!renderState.documentPath) return;
    const documentPath = $state.snapshot(renderState.documentPath);
    const imagesDirectory = $state.snapshot(globalSetupState.imagesDirectory);
    const dataDirectory = $state.snapshot(globalSetupState.dataDirectory);

    loadDocument().then(() => {
      invoke("run_extract_document_images_stage", {
        extractDocumentImagesStage: {
          documentPath,
          imagesDirectory,
          dataDirectory,
        },
      });
    });

    return () => {
      globalSetupState.clearState().then(() => {
        console.log("State cleaned");
      });
    };
  });

  $effect(() => {
    if (!renderState.documentProxy || !validPageNumber) return;
    renderState.scale;
    renderState.rotation;
    renderState.inProcessList.length;
    renderState.selectedPages.length;
    renderState.finishedDocumentsProcessStage.length;
    renderState.pageNumber = validPageNumber;
    untrack(() => loadPageQueue(validPageNumber));
  });

  $effect(() => {
    validPageNumber;
    return () => {
      tick().then(() => {
        renderState.isShowStatusCanvas = true;
      });
    };
  });

  const toggleStatusCanvas = () => {
    renderState.isShowStatusCanvas = !renderState.isShowStatusCanvas;
    if (renderState.isShowStatusCanvas) {
      loadPageQueue(validPageNumber);
    }
  };

  let canvasWrapper: HTMLDivElement;

  const isStatusToggleVisible = $derived(
    renderState.numPages &&
      (renderState.inProcessList.some((ip) =>
        ip.selectedPages.includes(validPageNumber),
      ) ||
        renderState.finishedDocumentsProcessStage.some((fd) =>
          fd.selectedPages.includes(validPageNumber),
        )),
  );

  const keyboardShortcuts = [
    { keys: ["Home"], description: "Primeira página" },
    { keys: ["End"], description: "Última página" },
    { keys: ["←"], description: "Página anterior" },
    { keys: ["→"], description: "Próxima página" },
    { keys: ["Shift", "←"], description: "Primeira página" },
    { keys: ["Shift", "→"], description: "Última página" },
    { keys: ["Space"], description: "Selecionar/Deselecionar página" },
    { keys: ["Backspace"], description: "Deselecionar última página" },
    { keys: ["Enter"], description: "Processar páginas selecionadas" },
    { keys: ["Tab"], description: "Próxima página" },
    { keys: ["Shift", "Tab"], description: "Página anterior" },
    { keys: ["Ctrl", "O"], description: "Abrir PDF" },
    { keys: ["Ctrl", "="], description: "Aumentar zoom" },
    { keys: ["Ctrl", "-"], description: "Diminuir zoom" },
    { keys: ["Ctrl", "←"], description: "Girar para esquerda" },
    { keys: ["Ctrl", "→"], description: "Girar para direita" },
    { keys: ["Shift", "↑"], description: "Aumentar zoom" },
    { keys: ["Shift", "↓"], description: "Diminuir zoom" },
    { keys: ["Ctrl", "Enter"], description: "Iniciar processamento final" },
    { keys: ["Esc"], description: "Cancelar edição de nome" },
  ];

  const workflowSchema = `
                        ┌─────────────┐
                        │  Abrir PDF  │
                        └─────┬───────┘
                              │
                              ▼
                      ┌───────────────────┐
                      │ Selecionar Páginas│
                      └─────────┬─────────┘
                                │
                                ▼
                      ┌───────────────────┐
                      │    Processar      │
                      │     Páginas       │
                      └─────────┬─────────┘
                                │
                                ▼
                      ┌───────────────────┐
                      │  Nome Sugerido    │
                      │   para Arquivo    │
                      └─────────┬─────────┘
                                │
                          ┌─────┴─────┐
                          ▼           ▼
                      ┌─────────┐ ┌─────────┐
                      │  Editar │ │  Salvar │
                      │  Nome   │ │  Nome   │
                      └─────────┘ └─────────┘
`;

  let showShortcuts = $state(false);

  const toggleShortcuts = () => {
    showShortcuts = !showShortcuts;
  };

  const runProcessDocuments = async () => {
    try {
      const result = await invoke<string>("extract_document_images", {
        paths: ["C:\\Users\\conta\\Downloads\\PDF_TESTE.pdf"],
        outputDir: "C:\\Users\\conta\\Downloads\\PDF_TESTE_IMAGES",
      });
      console.log("Process result:", result);
    } catch (error) {
      console.error("Error processing document:", error);
    }
  };

  const killUtility = () => {
    emit("kill-utility", { kill: true });
  };

  const cancelProcessing = () => {
    emit("cancel-processing", {});
  };

  $effect(() => {
    const unsubscribe1 = listen("utility-stdout", (data) => {
      console.log("Utility stdout:", data.payload);
    });
    const unsubscribe2 = listen("utility-terminated", (data) => {
      console.log("Utility terminated:", data.payload);
    });
    const unsubscribe3 = listen("utility-error", (data) => {
      console.error("Utility error:", data.payload);
    });
    const unsubscribe4 = listen<ProgressUpdate>("progress", (data) => {
      console.log("Progress:", data.payload);
      const {
        pages_processed,
        pages_to_process,
        estimated_seconds_remaining,
        extracted_page_numbers,
        total_document_pages,
      } = data.payload;
      console.log(
        `Processed ${pages_processed}/${pages_to_process} pages. Estimated time remaining: ${estimated_seconds_remaining} seconds`,
      );
      renderState.extractedPages = extracted_page_numbers;
      renderState.isExtractingImages =
        extracted_page_numbers.length !== total_document_pages;
    });
    const unsubscribe5 = listen<number>("total-extracted-pages", (event) => {
      console.log(`All ${event.payload} .webp files match the PDF pages.`);
      renderState.extractedPages = Array.from(
        { length: event.payload },
        (_, i) => i + 1,
      );
      renderState.isExtractingImages = false;
    });

    return () => {
      unsubscribe1.then((unsubscribe1) => unsubscribe1());
      unsubscribe2.then((unsubscribe2) => unsubscribe2());
      unsubscribe3.then((unsubscribe3) => unsubscribe3());
      unsubscribe4.then((unsubscribe4) => unsubscribe4());
      unsubscribe5.then((unsubscribe5) => unsubscribe5());
    };
  });
</script>

<svelte:window
  onmousedown={handleMouseDown}
  ondblclick={handleDoubleClick}
  onwheel={handleWheel}
  onkeydown={handleKeyDown}
/>

<div
  bind:this={component}
  class="relative h-full w-full border-4 {renderState.isActive
    ? 'border-primary'
    : 'border-accent'}"
>
  <div
    class="grid h-full w-full place-items-center overflow-auto bg-accent focus:outline-none"
  >
    {#if validPageNumber && !isPageAvailable && renderState.isExtractingImages}
      <div
        class="absolute text-primary inset-0 flex flex-col items-center justify-center space-y-4 text-center bg-accent/80 z-10"
      >
        <Loader2 class="h-8 w-8 animate-spin" />
        <p class="text-lg font-semibold">
          Extraindo imagem da página {validPageNumber}...
        </p>
        <p class="text-sm text-muted-foreground">
          Por favor, aguarde. Isso pode levar alguns instantes.
        </p>
      </div>
    {/if}
    <div bind:this={canvasWrapper} class="relative">
      <div bind:this={canvasContainer} class="relative">
        <canvas
          class={isPageAvailable ? "" : "pointer-events-none opacity-50"}
          bind:this={renderCanvas}
        ></canvas>
        <svg class="absolute left-0 top-0" bind:this={textLayer}></svg>
        <canvas
          bind:this={statusCanvas}
          class="pointer-events-none absolute left-0 top-0"
          style="display: {renderState.isShowStatusCanvas ? 'block' : 'none'};"
        ></canvas>
      </div>
      {#if isStatusToggleVisible}
        <Button
          tabindex={-1}
          size="icon"
          class="toggle-status-button absolute top-1/2 -right-12 -translate-y-1/2 transform"
          onclick={toggleStatusCanvas}
          aria-label={renderState.isShowStatusCanvas
            ? "Hide status overlay"
            : "Show status overlay"}
        >
          {#if renderState.isShowStatusCanvas}
            <EyeOff />
          {:else}
            <Eye />
          {/if}
        </Button>
      {/if}
    </div>
  </div>

  {#if renderState.numPages && renderState.selectedPages.length > 0}
    <div
      class="absolute left-4 top-4 flex w-1/2 select-none flex-wrap gap-1 overflow-x-auto"
    >
      {#each renderState.selectedPages as page}
        <Button
          class="aspect-square h-8 w-8 font-semibold"
          size="sm"
          onclick={() => handlePageNumberClick(page)}
        >
          {page}
        </Button>
      {/each}
    </div>
  {/if}
  <div class="absolute right-4 top-4 flex flex-col space-y-2">
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleRotateLeft}
      disabled={!renderState.numPages}
      aria-label="Rotate left"
    >
      <RotateCcw />
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleRotateRight}
      disabled={!renderState.numPages}
      aria-label="Rotate right"
    >
      <RotateCw />
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleZoomIn}
      disabled={!renderState.numPages || renderState.scale === MAX_SCALE}
      aria-label="Zoom in"
    >
      <ZoomIn />
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleZoomOut}
      disabled={!renderState.numPages || renderState.scale === MIN_SCALE}
      aria-label="Zoom out"
    >
      <ZoomOut />
    </Button>
  </div>

  <Button
    tabindex={-1}
    class="absolute bottom-4 left-4"
    size="icon"
    onclick={handleSelectPDF}
    aria-label="Open PDF"
  >
    <FolderOpen />
  </Button>

  <div
    class="absolute bottom-4 left-1/2 flex -translate-x-1/2 scale-90 transform items-center justify-center space-x-2 z-20"
  >
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleFirstPage}
      disabled={!renderState.numPages || renderState.pageNumber === 1}
      aria-label="First page"
    >
      <ChevronFirst />
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      onclick={handlePrevPage}
      disabled={!renderState.numPages || renderState.pageNumber === 1}
      aria-label="Previous page"
    >
      <ArrowLeft />
    </Button>
    <Input
      class="h-12 w-20 text-center text-2xl font-semibold text-primary focus:outline-none"
      tabindex={-1}
      type="number"
      bind:value={renderState.pageNumber}
      min="1"
      max={renderState.numPages}
      disabled={!renderState.numPages}
      aria-label="Page number"
    />
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleNextPage}
      disabled={!renderState.numPages ||
        renderState.pageNumber === renderState.numPages}
      aria-label="Next page"
    >
      <ArrowRight />
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      onclick={handleLastPage}
      disabled={!renderState.numPages ||
        renderState.pageNumber === renderState.numPages}
      aria-label="Last page"
    >
      <ChevronLast />
    </Button>
  </div>

  <div class="absolute bottom-4 right-4 flex flex-col space-y-2">
    <Dialog.Root bind:open={renderState.isDialogOpen}>
      <Dialog.Trigger
        tabindex={-1}
        disabled={!isPageAvailable ||
          renderState.selectedPages.length === 0 ||
          renderState.inProcessList.some((ip) =>
            ip.selectedPages.includes(renderState.pageNumber),
          )}
        class={buttonVariants({ size: "icon", className: "" })}
        aria-label="Process selected pages"
      >
        <FileCheck />
      </Dialog.Trigger>
      <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
          <Dialog.Title>
            {renderState.selectedPages.length > 1
              ? "Processar as páginas selecionadas?"
              : "Processar página selecionada?"}
          </Dialog.Title>
          <Dialog.Description>
            {selectedPagesText}
          </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
          <Button
            onclick={() => {
              handleProcessPages();
              renderState.isDialogOpen = false;
            }}
          >
            Processar
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>

    <Button
      tabindex={-1}
      size="icon"
      onclick={(e: MouseEvent) => {
        renderState.isActive && e.stopImmediatePropagation();
        handleSelectPage();
      }}
      disabled={!isPageAvailable ||
        renderState.inProcessList.some((ipl) =>
          ipl.selectedPages.includes(renderState.pageNumber),
        ) ||
        renderState.pageProcessStageSuccessList.some((pps) =>
          pps.selectedPages.includes(renderState.pageNumber),
        ) ||
        renderState.documentProcessStageErrorList.some((dpe) =>
          dpe.selectedPages.includes(renderState.pageNumber),
        ) ||
        renderState.documentProcessStageSuccessList.some((dps) =>
          dps.selectedPages.includes(renderState.pageNumber),
        ) ||
        renderState.documentProcessStageErrorList.some((dpe) =>
          dpe.selectedPages.includes(renderState.pageNumber),
        ) ||
        renderState.finishedDocumentsProcessStage.some((fdps) =>
          fdps.selectedPages.includes(renderState.pageNumber),
        )}
      aria-label={renderState.selectedPages.includes(renderState.pageNumber)
        ? "Deselect page"
        : "Select page"}
    >
      {#if renderState.selectedPages.includes(renderState.pageNumber)}
        <FileMinus />
      {:else}
        <FilePlus />
      {/if}
    </Button>
  </div>

  <div class="absolute top-1/2 left-4 -translate-y-1/2 z-10">
    <Button
      tabindex={-1}
      size="icon"
      variant="default"
      onclick={toggleShortcuts}
      aria-label={showShortcuts
        ? "Hide keyboard shortcuts"
        : "Show keyboard shortcuts"}
    >
      <Keyboard class="h-4 w-4" />
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      variant="default"
      onclick={runProcessDocuments}
    >
      Test
    </Button>
    <Button
      tabindex={-1}
      size="icon"
      variant="default"
      onclick={cancelProcessing}
    >
      Cancelar
    </Button>
  </div>

  {#if showShortcuts}
    <div
      class="absolute left-16 top-1/2 -translate-y-1/2 bg-background p-4 rounded-lg shadow-lg max-w-lg max-h-[80vh] overflow-y-auto z-20"
    >
      <h3 class="text-lg font-semibold mb-2 text-primary">Teclas de Atalho</h3>
      <div class="grid grid-cols-2 gap-2">
        {#each keyboardShortcuts as shortcut}
          <div class="flex items-center">
            <div class="flex-shrink-0">
              {#each shortcut.keys as key, index}
                <kbd class="kbd kbd-sm bg-secondary text-primary">{key}</kbd>
                {#if index < shortcut.keys.length - 1}
                  <span class="mx-1 text-gray-500">+</span>
                {/if}
              {/each}
            </div>
            <span class="ml-2 text-sm">{shortcut.description}</span>
          </div>
        {/each}
      </div>
      <Collapsible.Root>
        <Collapsible.Trigger
          tabindex={-1}
          class="flex items-center justify-between w-full text-md font-semibold mb-2 mt-4 text-primary"
          onclick={toggleWorkflow}
        >
          Fluxo de Trabalho
          {#if isWorkflowExpanded}
            <ChevronUp class="h-4 w-4" />
          {:else}
            <ChevronDown class="h-4 w-4" />
          {/if}
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="max-h-60 overflow-y-auto">
            <pre
              class="text-xs text-primary bg-secondary p-2 rounded-md overflow-x-auto whitespace-pre">
              {workflowSchema}
            </pre>
          </div>
        </Collapsible.Content>
      </Collapsible.Root>
    </div>
  {/if}
</div>
