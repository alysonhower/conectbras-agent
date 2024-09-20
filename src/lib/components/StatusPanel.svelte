<script lang="ts">
  import { getContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Ellipsis,
    Pencil,
    Check,
    X,
    History,
    RefreshCw,
    CheckCheck,
    FolderOpen,
  } from "lucide-svelte/icons";
  import { v4 as uuidv4 } from "uuid";

  import { Input } from "$lib/components/ui/input";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Separator } from "$lib/components/ui/separator";
  import * as Dialog from "$lib/components/ui/dialog";

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
    InProcessInstanceModel,
  } from "./processWorkflowContext.svelte";

  const renderState = globalSetupState.state;

  const allDocuments = $derived.by(() => {
    const inProcessLIst: InProcessInstanceModel[] = renderState.inProcessList;
    const pageProcessStageErrorList: PagePreprocessStageErrorModel[] =
      renderState.pageProcessStageErrorList;
    const pageProcessStageSuccessList: PagePreprocessStageSuccessModel[] =
      renderState.pageProcessStageSuccessList;
    const documentProcessStageErrorList: DocumentProcessStageErrorModel[] =
      renderState.documentProcessStageErrorList;
    const documentProcessStageSuccessList: DocumentProcessStageSuccessModel[] =
      renderState.documentProcessStageSuccessList;
    const finishedDocumentsProcessStage: FinishedDocumentProcessStageModel[] =
      renderState.finishedDocumentsProcessStage;
    return [
      ...inProcessLIst,
      ...pageProcessStageErrorList,
      ...pageProcessStageSuccessList,
      ...documentProcessStageErrorList,
      ...documentProcessStageSuccessList,
      ...finishedDocumentsProcessStage,
    ].sort((a, b) => {
      const aFirstPage =
        a instanceof InProcessInstanceModel
          ? Math.min(...a.stage.selectedPages)
          : Math.min(...a.selectedPages);
      const bFirstPage =
        b instanceof InProcessInstanceModel
          ? Math.min(...b.stage.selectedPages)
          : Math.min(...b.selectedPages);
      return aFirstPage - bFirstPage;
    });
  });

  $effect(() => {
    allDocuments;
    console.log("allDocuments changed: ", allDocuments);
  });

  const getTitle = (selectedPages: number[]) => {
    if (selectedPages.length === 1) {
      return `Página ${selectedPages[0]}`;
    } else if (selectedPages.length === 2) {
      return `Páginas ${selectedPages[0]} e ${selectedPages[1]}`;
    } else {
      return `Páginas ${selectedPages.slice(0, -1).join(", ")} e ${selectedPages.slice(-1)}.`;
    }
  };

  const getCardDescription = (
    document:
      | InProcessInstanceModel
      | PagePreprocessStageModel
      | PagePreprocessStageErrorModel
      | DocumentProcessStageModel
      | DocumentProcessStageErrorModel
      | FinishedDocumentProcessStageModel,
  ): string => {
    console.log("document: ", document.constructor.name);

    if (document instanceof InProcessInstanceModel) {
      if (document.stage instanceof PagePreprocessStageModel) {
        return `Pré-processando ${document.stage.selectedPages.length > 1 ? `páginas` : `página`}.`;
      } else if (document.stage instanceof PagePreprocessStageErrorModel) {
        return `Após erro ao pré-processar ${document.stage.selectedPages.length > 1 ? `as páginas` : `a página`}, tentando novamente.`;
      } else if (document.stage instanceof DocumentProcessStageModel) {
        return `Processando documento.`;
      } else if (document.stage instanceof DocumentProcessStageErrorModel) {
        return `Após erro ao processar o documento, tentando novamente.`;
      }
    } else if (document instanceof FinishedDocumentProcessStageModel) {
      return `Documento gerado com sucesso.`;
    } else if (document instanceof PagePreprocessStageErrorModel) {
      return `Erro ao pré-processar ${document.selectedPages.length > 1 ? `as páginas` : `a página`}. Se desejar, você pode tentar novamente.`;
    } else if (document instanceof DocumentProcessStageErrorModel) {
      return `Erro ao processar ${document.selectedPages.length > 1 ? `as páginas` : `a página`}. Se desejar, você pode tentar novamente.`;
    }
    return "";
  };

  const getCardContent = (
    document:
      | InProcessInstanceModel
      | PagePreprocessStageModel
      | PagePreprocessStageErrorModel
      | DocumentProcessStageModel
      | DocumentProcessStageErrorModel
      | FinishedDocumentProcessStageModel,
  ): string => {
    console.log("document: ", document.constructor.name);

    if (document instanceof InProcessInstanceModel) {
      if (document.stage instanceof PagePreprocessStageModel) {
        return `<p class="text-gray-700">Encaminhando ${document.stage.selectedPages.length > 1 ? `as images das` : `a imagem da`} ${getTitle(document.stage.selectedPages).toLowerCase()} para a IA.</p>`;
      } else if (document.stage instanceof PagePreprocessStageErrorModel) {
        return `<p class="text-red-600 font-semibold">Detalhes do erro: ${document.stage.errorMessage}.</p>`;
      } else if (document.stage instanceof DocumentProcessStageModel) {
        return `
        <p class="text-green-600 font-semibold mb-2">Pré-processamento concluído com sucesso!</p>
        <p class="mb-2">Gerando documento com ${document.stage.selectedPages.length} página(s).</p>
        <p class="mb-2">A IA sugeriu que o tipo do documento é <span class="font-semibold">"${document.stage.pagePreprocessStageResult.type_name}"</span> (abreviado fica <span class="font-semibold">"${document.stage.pagePreprocessStageResult.type_abbr}"</span>) e como nome do arquivo ela sugeriu <span class="font-semibold">"${document.stage.fileName}"</span>.</p>
        <p class="mb-2">Vou salvar-lo em <span class="font-semibold">${document.stage.dataDirectory}</span>.</p>
      `;
      } else {
        return `<p class="text-red-600 font-semibold">Detalhes do erro: ${document.stage.errorMessage}</p>`;
      }
    } else if (document instanceof FinishedDocumentProcessStageModel) {
      return `
      <p class="text-green-600 font-semibold mb-2">Documento com ${document.selectedPages.length} ${document.selectedPages.length > 1 ? `páginas` : `página`} processado com sucesso.</p>
      <p class="mb-1"><span class="font-semibold">Nome:</span> ${document.fileName}</p>
      <p class="mb-1"><span class="font-semibold">Tipo:</span> ${document.pagePreprocessStageResult.type_name} (${document.pagePreprocessStageResult.type_abbr})</p>
      <p class="mb-1"><span class="font-semibold">Resumo:</span> ${document.pagePreprocessStageResult.summary}</p>
      <p class="mb-1"><span class="font-semibold">Datas relevantes:</span> ${document.pagePreprocessStageResult.dates.map((d) => `${d.date} (${d.description})`).join(", ")}</p>
      <p class="mb-1"><span class="font-semibold">Documento salvo em:</span> ${document.documentPath}</p>
      <p><span class="font-semibold">Histórico de nomes:</span></p>
      <ul class="list-disc list-inside pl-4">
        ${document.fileNameHistory.map((name) => `<li>${name}</li>`).join("")}
      </ul>
    `;
    } else if (document instanceof PagePreprocessStageErrorModel) {
      return `<p class="text-red-600 font-semibold">Detalhes do erro: ${document.errorMessage}.</p>`;
    } else if (document instanceof DocumentProcessStageErrorModel) {
      return `<p class="text-red-600 font-semibold">Detalhes do erro: ${document.errorMessage}.</p>`;
    }
    return "";
  };

  const openInExplorer = async (documentPath: string) => {
    try {
      await invoke("open_in_explorer", { path: documentPath });
    } catch (error) {
      console.error(error);
    }
  };

  const updateFileName = async (
    fileName: string,
    id: string,
    documentPath: string,
  ) => {
    try {
      const newDocumentPath = await invoke<string>("run_update_file_name", {
        fileName,
        documentPath,
      });
      const finishedDocumentsProcessStage =
        globalSetupState.state.finishedDocumentsProcessStage;
      const index = finishedDocumentsProcessStage.findIndex(
        (document) => document.id === id,
      );
      const finishedDocument = finishedDocumentsProcessStage[index];
      let newFileNameHistory = finishedDocument.fileNameHistory;
      if (!newFileNameHistory.includes(fileName)) {
        newFileNameHistory.push(fileName);
      }
      const newFinishedDocument = new FinishedDocumentProcessStageModel(
        finishedDocument.id,
        finishedDocument.selectedPages,
        finishedDocument.dataDirectory,
        finishedDocument.imagesDirectory,
        finishedDocument.pagePreprocessStageResult,
        newDocumentPath,
        fileName,
        newFileNameHistory,
        finishedDocument.pageNumberPrefix,
      );
      if (index !== -1) {
        finishedDocumentsProcessStage.splice(index, 1);
      }
      finishedDocumentsProcessStage.push(newFinishedDocument);
    } catch (error) {
      console.error(error);
    }
  };

  interface NewFileNames {
    [key: string]: string;
  }

  interface VerifiedDocuments {
    [key: string]: boolean;
  }

  let newFileNames = $state<NewFileNames>({});

  let verifiedDocuments = $state<VerifiedDocuments>({});

  const handleRemovePageNumberPrefix = async (
    finishedDocument: FinishedDocumentProcessStageModel,
  ) => {
    try {
      const fileName =
        finishedDocument.pagePreprocessStageResult.suggested_file_name;
      const documentPath = finishedDocument.documentPath;
      await updateFileName(fileName, finishedDocument.id, documentPath);
      verifiedDocuments[finishedDocument.id] = true;
    } catch (error) {
      console.error(error);
    }
  };

  $effect(() => {
    newFileNames;
    console.log("newFileNames changed: ", newFileNames);
  });

  const handleRetryPagePreprocessStage = async (
    pagePreprocessStageError: PagePreprocessStageErrorModel,
  ) => {
    const index = renderState.pageProcessStageErrorList.findIndex(
      (error) => error.id === pagePreprocessStageError.id,
    );
    if (index !== -1) {
      renderState.pageProcessStageErrorList.splice(index, 1);
    }

    const pagePreprocessStageModel = new PagePreprocessStageModel(
      uuidv4(),
      pagePreprocessStageError.selectedPages,
      pagePreprocessStageError.dataDirectory,
      pagePreprocessStageError.imagesDirectory,
    );

    const pageProcessStageInstance = new InProcessInstanceModel(
      pagePreprocessStageModel,
    );
    renderState.inProcessList.push(pageProcessStageInstance);

    try {
      const runPagePreprocessStage = await invoke("run_page_preprocess_stage", {
        pagePreprocessStage: pagePreprocessStageModel,
      });

      const runPagePreprocessStageSuccess =
        runPagePreprocessStage as PagePreprocessStageSuccessModel;

      const pagePreprocessStageSuccess = new PagePreprocessStageSuccessModel(
        runPagePreprocessStageSuccess.id,
        runPagePreprocessStageSuccess.selectedPages,
        runPagePreprocessStageSuccess.dataDirectory,
        runPagePreprocessStageSuccess.imagesDirectory,
        runPagePreprocessStageSuccess.pagePreprocessStageResult,
        runPagePreprocessStageSuccess.pageNumberPrefix,
      );

      const ppsIndex = renderState.inProcessList.findIndex(
        (pps) => pps.stage.id === pagePreprocessStageSuccess.id,
      );

      if (ppsIndex !== -1) {
        renderState.inProcessList.splice(ppsIndex, 1);
      }

      const fileName =
        pagePreprocessStageSuccess.pagePreprocessStageResult
          .suggested_file_name;

      const documentProcessStageModel = new DocumentProcessStageModel(
        pagePreprocessStageSuccess.id,
        pagePreprocessStageSuccess.selectedPages,
        pagePreprocessStageSuccess.dataDirectory,
        pagePreprocessStageSuccess.imagesDirectory,
        pagePreprocessStageSuccess.pagePreprocessStageResult,
        globalSetupState.documentClonePath,
        fileName,
        pagePreprocessStageSuccess.pageNumberPrefix,
      );

      const documentProcessStageInstance = new InProcessInstanceModel(
        documentProcessStageModel,
      );
      renderState.inProcessList.push(documentProcessStageInstance);

      try {
        const documentProcessStage = await invoke(
          "run_document_process_stage",
          {
            documentProcessStage: documentProcessStageModel,
          },
        );

        const documentProcessStageSuccessModel =
          documentProcessStage as DocumentProcessStageSuccessModel;

        const dpsIndex = renderState.inProcessList.findIndex(
          (pps) => pps.stage.id === documentProcessStageSuccessModel.id,
        );
        if (dpsIndex !== -1) {
          renderState.inProcessList.splice(dpsIndex, 1);
        }

        const finishedDocumentProcessStage =
          new FinishedDocumentProcessStageModel(
            documentProcessStageSuccessModel.id,
            documentProcessStageSuccessModel.selectedPages,
            documentProcessStageSuccessModel.dataDirectory,
            documentProcessStageSuccessModel.imagesDirectory,
            documentProcessStageSuccessModel.pagePreprocessStageResult,
            documentProcessStageSuccessModel.documentPath,
            documentProcessStageSuccessModel.fileName,
            [documentProcessStageSuccessModel.fileName],
            documentProcessStageSuccessModel.pageNumberPrefix,
          );

        renderState.finishedDocumentsProcessStage.push(
          finishedDocumentProcessStage,
        );
      } catch (error) {
        const documentProcessStageError =
          error as DocumentProcessStageErrorModel;

        const dpsIndex = renderState.inProcessList.findIndex(
          (pps) => pps.stage.id === documentProcessStageError.id,
        );
        if (dpsIndex !== -1) {
          renderState.inProcessList.splice(dpsIndex, 1);
        }

        const documentProcessStageErrorModel =
          new DocumentProcessStageErrorModel(
            documentProcessStageError.id,
            documentProcessStageError.selectedPages,
            documentProcessStageError.dataDirectory,
            documentProcessStageError.imagesDirectory,
            documentProcessStageError.pagePreprocessStageResult,
            documentProcessStageError.documentPath,
            documentProcessStageError.fileName,
            documentProcessStageError.errorMessage,
            pagePreprocessStageSuccess.pageNumberPrefix,
          );

        renderState.documentProcessStageErrorList.push(
          documentProcessStageErrorModel,
        );
      }
    } catch (error) {
      const pagePreprocessStageError = error as PagePreprocessStageErrorModel;

      const ppsIndex = renderState.inProcessList.findIndex(
        (pps) => pps.stage.id === pagePreprocessStageError.id,
      );
      if (ppsIndex !== -1) {
        renderState.inProcessList.splice(ppsIndex, 1);
      }

      const pagePreprocessStageErrorModel = new PagePreprocessStageErrorModel(
        pagePreprocessStageError.id,
        pagePreprocessStageError.selectedPages,
        pagePreprocessStageError.dataDirectory,
        pagePreprocessStageError.imagesDirectory,
        pagePreprocessStageError.errorMessage,
      );

      renderState.pageProcessStageErrorList.push(pagePreprocessStageErrorModel);
    }
  };

  const handleRetryDocumentProcessStage = async (
    documentProcessStageError: DocumentProcessStageErrorModel,
  ) => {
    const index = renderState.documentProcessStageErrorList.findIndex(
      (error) => error.id === documentProcessStageError.id,
    );
    if (index !== -1) {
      renderState.documentProcessStageErrorList.splice(index, 1);
    }

    const documentProcessStageModel = new DocumentProcessStageModel(
      uuidv4(),
      documentProcessStageError.selectedPages,
      documentProcessStageError.dataDirectory,
      documentProcessStageError.imagesDirectory,
      documentProcessStageError.pagePreprocessStageResult,
      documentProcessStageError.documentPath,
      documentProcessStageError.fileName,
      documentProcessStageError.pageNumberPrefix,
    );

    const documentProcessStageInstance = new InProcessInstanceModel(
      documentProcessStageModel,
    );
    renderState.inProcessList.push(documentProcessStageInstance);

    try {
      const documentProcessStage = await invoke("run_document_process_stage", {
        documentProcessStage: documentProcessStageModel,
      });

      const documentProcessStageSuccessModel =
        documentProcessStage as DocumentProcessStageSuccessModel;

      const dpsIndex = renderState.inProcessList.findIndex(
        (pps) => pps.stage.id === documentProcessStageSuccessModel.id,
      );
      if (dpsIndex !== -1) {
        renderState.inProcessList.splice(dpsIndex, 1);
      }

      const finishedDocumentProcessStage =
        new FinishedDocumentProcessStageModel(
          documentProcessStageSuccessModel.id,
          documentProcessStageSuccessModel.selectedPages,
          documentProcessStageSuccessModel.dataDirectory,
          documentProcessStageSuccessModel.imagesDirectory,
          documentProcessStageSuccessModel.pagePreprocessStageResult,
          documentProcessStageSuccessModel.documentPath,
          documentProcessStageSuccessModel.fileName,
          [documentProcessStageSuccessModel.fileName],
          documentProcessStageSuccessModel.pageNumberPrefix,
        );

      renderState.finishedDocumentsProcessStage.push(
        finishedDocumentProcessStage,
      );
    } catch (error) {
      const documentProcessStageError = error as DocumentProcessStageErrorModel;

      const dpsIndex = renderState.inProcessList.findIndex(
        (pps) => pps.stage.id === documentProcessStageError.id,
      );
      if (dpsIndex !== -1) {
        renderState.inProcessList.splice(dpsIndex, 1);
      }

      const documentProcessStageErrorModel = new DocumentProcessStageErrorModel(
        documentProcessStageError.id,
        documentProcessStageError.selectedPages,
        documentProcessStageError.dataDirectory,
        documentProcessStageError.imagesDirectory,
        documentProcessStageError.pagePreprocessStageResult,
        documentProcessStageError.documentPath,
        documentProcessStageError.fileName,
        documentProcessStageError.errorMessage,
        documentProcessStageError.pageNumberPrefix,
      );

      renderState.documentProcessStageErrorList.push(
        documentProcessStageErrorModel,
      );
    }
  };

  $effect(() => {
    allDocuments.forEach((doc) => {
      if (
        doc instanceof FinishedDocumentProcessStageModel &&
        !newFileNames[doc.id]
      ) {
        newFileNames[doc.id] = doc.fileName;
      }
    });
  });
</script>

<div class="w-full h-full overflow-y-auto p-4 space-y-4">
  {#each allDocuments as document (document.id)}
    {@const title = `${getTitle(document instanceof InProcessInstanceModel ? document.stage.selectedPages : document.selectedPages)}`}
    <Card.Root
      class="w-full max-h-[50vh] flex flex-col {verifiedDocuments[document.id]
        ? 'bg-green-100'
        : ''}"
    >
      <Card.Header>
        <Card.Title class="text-lg font-semibold break-all">{title}</Card.Title>

        <Card.Description class="text-sm text-gray-500 break-all">
          {getCardDescription(document)}
        </Card.Description>
      </Card.Header>
      <Card.Content class="text-sm flex-grow overflow-y-auto">
        <div class="prose prose-sm max-w-none break-all">
          {@html getCardContent(document)}
        </div>
      </Card.Content>
      <Card.Footer>
        {#if document instanceof FinishedDocumentProcessStageModel}
          <div class="flex flex-col space-y-2 w-full mt-2">
            <Input
              bind:value={newFileNames[document.id]}
              placeholder={document.fileName}
            />
            <div class="flex justify-end space-x-2 w-full">
              <Button
                disabled={!newFileNames[document.id] ||
                  newFileNames[document.id] === document.fileName}
                onclick={async () =>
                  await updateFileName(
                    newFileNames[document.id] || document.fileName,
                    document.id,
                    document.documentPath,
                  )}
              >
                <Pencil class="mr-2 h-4 w-4" />Editar nome
              </Button>
              {#if verifiedDocuments[document.id]}
                <Button onclick={() => openInExplorer(document.documentPath)}>
                  <FolderOpen class="mr-2 h-4 w-4" />Abrir no Explorer
                </Button>
              {/if}
              <Button
                onclick={async () =>
                  await handleRemovePageNumberPrefix(document)}
                disabled={verifiedDocuments[document.id]}
              >
                <CheckCheck class="mr-2 h-4 w-4" />

                {verifiedDocuments[document.id] ? "Verificado" : "Verificar"}
              </Button>
            </div>
          </div>
        {:else if document instanceof PagePreprocessStageErrorModel}
          <div class="flex justify-end w-full">
            <Button
              onclick={async () =>
                await handleRetryPagePreprocessStage(document)}
            >
              <RefreshCw class="mr-2 h-4 w-4" />Tentar novamente
            </Button>
          </div>
        {:else if document instanceof DocumentProcessStageErrorModel}
          <div class="flex justify-end w-full">
            <Button
              onclick={async () =>
                await handleRetryDocumentProcessStage(document)}
            >
              <RefreshCw class="mr-2 h-4 w-4" />Tentar novamente
            </Button>
          </div>
        {/if}
      </Card.Footer>
    </Card.Root>
  {/each}
</div>
