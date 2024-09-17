<script lang="ts">
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

  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Separator } from "$lib/components/ui/separator";
  import * as Dialog from "$lib/components/ui/dialog";
  import {
    globalSetupState,
    type PagePreprocessStageResult,
    type PagePreprocessStage,
    type DocumentProcessStage,
    type FinishedDocumentProcessStage,
    type PagePreprocessStageError,
    type PagePreprocessStageSuccess,
  } from "./processWorkflowContext.svelte";

  const renderState = globalSetupState.state;

  interface StatusPanelState {
    time: Date;
    editingDocumentId: string | undefined;
    editedFileName: string | undefined;
    showHistoryMap: Map<string, boolean>;
    isDropdownOpenMap: Map<string, boolean>;
    historyHoverTimeoutMap: Map<string, NodeJS.Timeout>;
    confirmProcessDialogOpenMap: Map<string, boolean>;
  }

  const statusPanel = $state<StatusPanelState>({
    time: new Date(),
    editingDocumentId: undefined,
    editedFileName: undefined,
    showHistoryMap: new Map<string, boolean>(),
    isDropdownOpenMap: new Map<string, boolean>(),
    historyHoverTimeoutMap: new Map<string, NodeJS.Timeout>(),
    confirmProcessDialogOpenMap: new Map<string, boolean>(),
  });

  const formatPagesText = (pages: number[]): string => {
    if (pages.length === 1) return pages[0].toString();
    if (pages.length === 2) return `${pages[0]} e ${pages[1]}`;
    return `${pages.slice(0, -1).join(", ")} e ${pages[pages.length - 1]}`;
  };

  const formatDurationText = (startTime: number, endTime?: number): string => {
    const duration = (endTime || Date.now()) - startTime;
    const seconds = Math.floor(duration / 1000);
    const minutes = Math.floor(seconds / 60);
    return minutes > 0 ? `${minutes}m ${seconds % 60}s` : `${seconds}s`;
  };

  const translateStatusText = (status: string): string => {
    const statusMap = {
      pending: "pendente",
      processing: "processando",
      completed: "concluído",
      error: "erro",
    } as const;
    return statusMap[status as keyof typeof statusMap] || "desconhecido";
  };

  type AllDocumentTypes =
    | (PagePreprocessStage & { listType: "pageProcessStage" })
    | (DocumentProcessStage & { listType: "documentProcessStage" })
    | (FinishedDocumentProcessStage & { listType: "finishedDocumentStage" })
    | (PagePreprocessStageError & { listType: "pageProcessStageError" });

  const allDocuments = $derived(
    [
      ...renderState.pagesProcessStage.map((doc) => ({
        ...doc,
        listType: "pageProcessStage" as const,
      })),
      ...renderState.documentsProcessStage.map((doc) => ({
        ...doc,
        listType: "documentProcessStage" as const,
      })),
      ...renderState.finishedDocumentsProcessStage.map((doc) => ({
        ...doc,
        listType: "finishedDocumentStage" as const,
      })),
      ...renderState.pagesProcessStageErrors.map((doc) => ({
        ...doc,
        listType: "pageProcessStageError" as const,
      })),
    ].sort(
      (a, b) => Math.min(...a.selectedPages) - Math.min(...b.selectedPages),
    ),
  );

  const setMapValue = (map: Map<string, any>, id: string, value: any) => {
    return new Map(map).set(id, value);
  };

  const handleHistoryInteraction = (id: string, isEnter: boolean) => {
    clearTimeout(statusPanel.historyHoverTimeoutMap.get(id));
    if (isEnter) {
      statusPanel.showHistoryMap = setMapValue(
        statusPanel.showHistoryMap,
        id,
        true,
      );
      statusPanel.isDropdownOpenMap = setMapValue(
        statusPanel.isDropdownOpenMap,
        id,
        true,
      );
    } else {
      const timeout = setTimeout(() => {
        statusPanel.showHistoryMap = setMapValue(
          statusPanel.showHistoryMap,
          id,
          false,
        );
        statusPanel.isDropdownOpenMap = setMapValue(
          statusPanel.isDropdownOpenMap,
          id,
          false,
        );
      }, 300);
      statusPanel.historyHoverTimeoutMap = setMapValue(
        statusPanel.historyHoverTimeoutMap,
        id,
        timeout,
      );
    }
  };

  const handleEditButtonInteraction = (id: string, isEnter: boolean) => {
    if (isEnter) {
      statusPanel.showHistoryMap = setMapValue(
        statusPanel.showHistoryMap,
        id,
        true,
      );
    } else {
      const timeout = setTimeout(() => {
        if (!statusPanel.isDropdownOpenMap.get(id)) {
          statusPanel.showHistoryMap = setMapValue(
            statusPanel.showHistoryMap,
            id,
            false,
          );
        }
      }, 300);
      statusPanel.historyHoverTimeoutMap = setMapValue(
        statusPanel.historyHoverTimeoutMap,
        id,
        timeout,
      );
    }
  };

  const startEditing = (
    document:
      | (DocumentProcessStage & { listType: "documentProcessStage" })
      | (FinishedDocumentProcessStage & { listType: "finishedDocumentStage" }),
  ) => {
    statusPanel.editingDocumentId = document.id;
    statusPanel.editedFileName = document.fileName;
  };

  const saveFileName = async (
    document:
      | (DocumentProcessStage & { listType: "documentProcessStage" })
      | (FinishedDocumentProcessStage & { listType: "finishedDocumentStage" }),
  ) => {
    if (!statusPanel.editedFileName?.trim()) return;

    try {
      let updatedDocumentInfo: DocumentProcessStage;
      if (document.listType === "finishedDocumentStage") {
        console.log(
          "Renaming finished document:",
          "json path: " + document.imagesDirectory,
          "new name: " + statusPanel.editedFileName,
        );
        updatedDocumentInfo = await invoke("rename_finished_document", {
          oldPath: document.imagesDirectory,
          newName: statusPanel.editedFileName,
        });
      } else {
        updatedDocumentInfo = await invoke("update_file_name", {
          path: document.imagesDirectory,
          name: statusPanel.editedFileName,
        });
      }
      console.log("Updated document info:", updatedDocumentInfo);

      if (!document.fileNameHistory) {
        document.fileNameHistory = [];
      }
      if (!document.fileNameHistory.includes(statusPanel.editedFileName)) {
        document.fileNameHistory.push(statusPanel.editedFileName);
      }

      if (document.listType === "finishedDocumentStage") {
        renderState.finishedDocumentsProcessStage =
          renderState.finishedDocumentsProcessStage.map((doc) =>
            doc.id === document.id
              ? {
                  ...doc,
                  fileName: updatedDocumentInfo.fileName,
                  info: updatedDocumentInfo,
                }
              : doc,
          );
      } else {
        renderState.documentsProcessStage =
          renderState.documentsProcessStage.map((doc) =>
            doc.id === document.id
              ? {
                  ...doc,
                  fileName: updatedDocumentInfo.fileName,
                  info: updatedDocumentInfo,
                }
              : doc,
          );
      }

      statusPanel.showHistoryMap = setMapValue(
        statusPanel.showHistoryMap,
        document.id,
        false,
      );
    } catch (error) {
      console.error("Error updating file name:", error);
    } finally {
      statusPanel.editingDocumentId = undefined;
    }
  };

  const cancelEditing = () => {
    statusPanel.editingDocumentId = undefined;
    statusPanel.editedFileName = undefined;
  };

  const handleKeyDown = async (
    event: KeyboardEvent,
    document:
      | (DocumentProcessStage & { listType: "documentProcessStage" })
      | (FinishedDocumentProcessStage & { listType: "finishedDocumentStage" }),
  ) => {
    if (event.key === "Enter") await saveFileName(document);
    else if (event.key === "Escape") cancelEditing();
  };

  const elapsedTimes = $derived(
    renderState.pagesProcessStage.map((page) => ({
      id: page.id,
      elapsed: formatDurationText(page.startTime, statusPanel.time.getTime()),
    })),
  );

  const removeFromProcessed = (document: AllDocumentTypes) => {
    renderState.documentsProcessStage =
      renderState.documentsProcessStage.filter((doc) => doc.id !== document.id);
  };

  const addToProcessing = (document: AllDocumentTypes) => {
    renderState.pagesProcessStage = [
      ...renderState.pagesProcessStage,
      { ...document, startTime: Date.now() },
    ];
  };

  const isProcessedOrFinished = (
    doc: AllDocumentTypes,
  ): doc is (DocumentProcessStage | FinishedDocumentProcessStage) & {
    listType: "documentProcessStage" | "finishedDocumentStage";
  } =>
    doc.listType === "documentProcessStage" ||
    doc.listType === "finishedDocumentStage";

  const handleFinalPipeline = async (document: AllDocumentTypes) => {
    if (!isProcessedOrFinished(document)) return;
    setConfirmProcessDialogOpen(document.id, false);

    removeFromProcessed(document);
    addToProcessing(document);

    try {
      await invoke("final_pipeline", {
        documentInfo: document.preprocessPagesStageResult,
      });
      document.endTime = Date.now();
      renderState.finishedDocumentsProcessStage = [
        ...renderState.finishedDocumentsProcessStage,

        document,
      ];
    } catch (error) {
      console.error("Error in final pipeline:", error);
      renderState.documentsProcessStage = [
        ...renderState.documentsProcessStage,
        document,
      ];
    } finally {
      renderState.pagesProcessStage = renderState.pagesProcessStage.filter(
        (page) => page.id !== document.id,
      );
    }
  };

  const setConfirmProcessDialogOpen = (id: string, isOpen: boolean) => {
    statusPanel.confirmProcessDialogOpenMap = new Map(
      statusPanel.confirmProcessDialogOpenMap,
    ).set(id, isOpen);
  };

  const isConfirmProcessDialogOpen = (id: string) => {
    return statusPanel.confirmProcessDialogOpenMap.get(id) || false;
  };

  const handleRetry = async (document: AllDocumentTypes) => {
    if (document.listType !== "documentProcessStage") return; // Only processed documents can be retried

    const pageProcessStage: PagePreprocessStage = {
      id: uuidv4(),
      dataDirectory: document.dataDirectory,
      imagesDirectory: document.imagesDirectory,
      selectedPages: document.selectedPages,
      startTime: Date.now(),
    };

    renderState.pagesProcessStage = [
      ...renderState.pagesProcessStage,
      pageProcessStage,
    ];

    renderState.documentsProcessStage =
      renderState.documentsProcessStage.filter((doc) => doc.id !== document.id);

    try {
      const preprocessPagesStateResult: PagePreprocessStageResult =
        await invoke("generate_file_name", {
          processDocumentProcessStage: document,
        });

      const pageProcessStageSuccess: PagePreprocessStageSuccess = {
        ...pageProcessStage,
        endTime: Date.now(),
        elapsedTime: 0,
        preprocessPagesStageResult: preprocessPagesStateResult,
      };

      console.log("pageProcessStageSuccess:", pageProcessStageSuccess);

      const processDocumentStage: DocumentProcessStage = {
        ...pageProcessStageSuccess,
        fileName: preprocessPagesStateResult.suggested_file_name,
        fileNameHistory: [preprocessPagesStateResult.suggested_file_name],
        documentPath: document.documentPath,
      };

      renderState.documentsProcessStage = [
        ...renderState.documentsProcessStage,
        processDocumentStage,
      ];

      console.log("generate_file_name:", preprocessPagesStateResult);
    } catch (error) {
      console.error("Error in generate_file_name:", error);
    } finally {
      renderState.pagesProcessStage = renderState.pagesProcessStage.filter(
        (pp) => pp.id !== pageProcessStage.id,
      );
    }
  };

  const isCurrentPage = (document: AllDocumentTypes) => {
    return document.selectedPages.includes(renderState.pageNumber);
  };

  const handleGlobalKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey && event.key === "Enter") {
      const currentDocument = allDocuments.find(
        (doc) => isCurrentPage(doc) && isProcessedOrFinished(doc),
      );
      if (currentDocument) {
        event.preventDefault();
        setConfirmProcessDialogOpen(currentDocument.id, true);
      }
    }
  };

  const openInExplorer = async (document: FinishedDocumentProcessStage) => {
    try {
      const fullPath =
        document.imagesDirectory
          .replace(/(.+)-data(\\|\/)[^\\\/]+\.json$/, "$1-data\\done\\")
          .replace(/\//g, "\\") +
        document.fileName +
        ".pdf";
      console.log(fullPath);
      await invoke("open_in_explorer", { path: fullPath });
    } catch (error) {
      console.error("Error opening file in explorer:", error);
    }
  };

  const getRingStyle = (document: AllDocumentTypes): string => {
    if (document.listType === "pageProcessStage") {
      return "ring-4 ring-[rgba(0,128,0,1)]";
    } else if (document.listType === "documentProcessStage") {
      return "ring-4 ring-[rgba(186,79,125,1)]";
    } else {
      return "ring-4 ring-[rgba(255,165,0,0.8)]";
    }
  };

  $effect(() => {
    const interval = setInterval(() => (statusPanel.time = new Date()), 1000);
    window.addEventListener("keydown", handleGlobalKeydown);
    return () => {
      clearInterval(interval);
      window.removeEventListener("keydown", handleGlobalKeydown);
    };
  });
</script>

<div
  class="flex h-full w-full flex-col overflow-y-auto justify-between bg-accent p-4"
>
  <h1 class="text-base font-semibold text-primary mb-4 break-all">
    Status do processamento
  </h1>
  <div class="flex h-full w-full flex-col gap-4">
    {#each allDocuments as document (document.id)}
      <Card.Root
        class="p-3 max-h-[50vh] flex flex-col {isCurrentPage(document)
          ? getRingStyle(document)
          : ''}"
        tabindex={isCurrentPage(document) ? 0 : -1}
      >
        <Card.Header class="pb-1 flex-shrink-0">
          <Card.Title class="flex flex-col gap-1 text-sm">
            <span class="font-semibold text-primary break-all">
              {#if document.listType === "pageProcessStage"}
                Processando ({document.selectedPages.length > 1
                  ? "páginas"
                  : "página"}: {formatPagesText(document.selectedPages)})
              {:else if document.listType === "documentProcessStage"}
                Documento Finalizado:
              {:else}
                Nome sugerido (página: {formatPagesText(
                  document.selectedPages,
                )}):
              {/if}
            </span>
            {#if document.listType !== "pageProcessStage" && document.listType !== "pageProcessStageError"}
              {#if statusPanel.editingDocumentId === document.id}
                <div class="w-full h-full space-y-1">
                  <!-- svelte-ignore a11y_autofocus -->
                  <div
                    class="w-full p-2 border rounded-md font-semibold text-sm focus:outline-none focus:ring-1 focus:ring-primary break-all min-h-[1.5em] max-h-[50vh] overflow-y-auto"
                    contenteditable="true"
                    bind:textContent={statusPanel.editedFileName}
                    onkeydown={(e) => handleKeyDown(e, document)}
                    onfocus={(e) => {
                      if (e.target instanceof Node) {
                        const range = window.document.createRange();
                        range.selectNodeContents(e.target);
                        const selection = window.getSelection();
                        selection?.removeAllRanges();
                        selection?.addRange(range);
                      }
                    }}
                    role="textbox"
                    tabindex="0"
                    autofocus
                  ></div>
                  <div class="flex justify-end space-x-1">
                    <Button
                      size="icon"
                      variant="default"
                      onclick={() => saveFileName(document)}
                      disabled={!statusPanel.editedFileName?.trim()}
                      class="h-7 w-7"
                    >
                      <Check class="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      size="icon"
                      variant="default"
                      onclick={cancelEditing}
                      class="h-7 w-7"
                    >
                      <X class="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              {:else}
                <span class="break-all w-full font-semibold text-sm mb-1">
                  {document.fileName}
                </span>
                <div class="flex justify-end space-x-1 relative">
                  <div class="relative">
                    {#if document.fileNameHistory && document.fileNameHistory.length > 1}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="absolute right-full transition-all duration-300 ease-in-out transform"
                        class:translate-x-[100%]={!statusPanel.showHistoryMap.get(
                          document.id,
                        )}
                        class:opacity-0={!statusPanel.showHistoryMap.get(
                          document.id,
                        )}
                        class:pointer-events-none={!statusPanel.showHistoryMap.get(
                          document.id,
                        )}
                        onmouseenter={() =>
                          handleHistoryInteraction(document.id, true)}
                        onmouseleave={() =>
                          handleHistoryInteraction(document.id, false)}
                      >
                        <DropdownMenu.Root
                          open={statusPanel.isDropdownOpenMap.get(document.id)}
                        >
                          <DropdownMenu.Trigger asChild let:builder>
                            <Button
                              size="icon"
                              variant="default"
                              builders={[builder]}
                              class="h-7 w-7 mr-1"
                            >
                              <History class="h-3.5 w-3.5" />
                            </Button>
                          </DropdownMenu.Trigger>
                          {#key document.fileNameHistory}
                            <DropdownMenu.Content
                              class="w-96 max-w-[90vw] max-h-60 overflow-hidden flex flex-col"
                              onmouseenter={() =>
                                handleHistoryInteraction(document.id, true)}
                              onmouseleave={() =>
                                handleHistoryInteraction(document.id, false)}
                            >
                              <div
                                class="sticky top-0 bg-background z-10 py-1.5 px-2"
                              >
                                <DropdownMenu.Label
                                  >Histórico de nomes</DropdownMenu.Label
                                >
                                <DropdownMenu.Separator />
                              </div>
                              <div class="overflow-y-auto">
                                {#each document.fileNameHistory
                                  .slice()
                                  .reverse() as historyItem}
                                  <DropdownMenu.Item
                                    onclick={() => {
                                      statusPanel.editedFileName = historyItem;
                                      saveFileName(document);
                                      statusPanel.isDropdownOpenMap =
                                        setMapValue(
                                          statusPanel.isDropdownOpenMap,
                                          document.id,
                                          false,
                                        );
                                    }}
                                    class="break-all cursor-pointer"
                                  >
                                    {historyItem}
                                    {historyItem === document.fileName
                                      ? " (nome atual)"
                                      : ""}
                                  </DropdownMenu.Item>
                                {/each}
                              </div>
                            </DropdownMenu.Content>
                          {/key}
                        </DropdownMenu.Root>
                      </div>
                    {/if}
                    <Button
                      size="icon"
                      variant="default"
                      onclick={() => startEditing(document)}
                      class="h-7 w-7 relative z-10"
                      onmouseenter={() =>
                        handleEditButtonInteraction(document.id, true)}
                      onmouseleave={() =>
                        handleEditButtonInteraction(document.id, false)}
                    >
                      <Pencil class="h-3.5 w-3.5" />
                    </Button>
                    {#if document.listType === "documentProcessStage"}
                      <Button
                        size="icon"
                        variant="default"
                        onclick={() => openInExplorer(document)}
                        class="h-7 w-7"
                      >
                        <FolderOpen class="h-3.5 w-3.5" />
                      </Button>
                    {:else if document.listType === "finishedDocumentStage"}
                      <Dialog.Root
                        open={isConfirmProcessDialogOpen(document.id)}
                        onOpenChange={(open) =>
                          setConfirmProcessDialogOpen(document.id, open)}
                      >
                        <Dialog.Trigger
                          tabindex={-1}
                          class={buttonVariants({
                            size: "icon",
                            className: "h-7 w-7",
                          })}
                          aria-label="Process selected pages"
                        >
                          <CheckCheck class="h-3.5 w-3.5" />
                        </Dialog.Trigger>
                        <Dialog.Content class="sm:max-w-[425px]">
                          <Dialog.Header>
                            <Dialog.Title>
                              {document.selectedPages.length > 1
                                ? "Processar as páginas selecionadas?"
                                : "Processar página selecionada?"}
                            </Dialog.Title>
                            <Dialog.Description>
                              Você está prestes a processar {document
                                .selectedPages.length > 1
                                ? "as páginas"
                                : "a página"}
                              {formatPagesText(document.selectedPages)}. Deseja
                              continuar?
                            </Dialog.Description>
                          </Dialog.Header>
                          <Dialog.Footer>
                            <Button
                              onclick={() => handleFinalPipeline(document)}
                            >
                              Processar
                            </Button>
                          </Dialog.Footer>
                        </Dialog.Content>
                      </Dialog.Root>
                    {/if}
                  </div>
                </div>
              {/if}
            {/if}
          </Card.Title>
        </Card.Header>
        <Separator class="mt-1.5 bg-secondary mb-3 flex-shrink-0" />
        <Card.Content class="space-y-1 text-xs overflow-y-auto">
          <p>
            <span class="font-semibold text-primary">Status:</span>
            {document.listType === "finishedDocumentStage"
              ? "Finalizado"
              : translateStatusText(document.listType)}
          </p>
          {#if document.listType === "pageProcessStage"}
            <p>
              <span class="font-semibold text-primary">Tempo decorrido:</span>
              {#key statusPanel.time}<span
                  >{elapsedTimes.find((t) => t.id === document.id)
                    ?.elapsed}</span
                >{/key}
            </p>
          {:else}
            <p>
              <span class="font-semibold text-primary"
                >Tempo de processamento:</span
              >
              {formatDurationText(document.startTime, document.endTime)}
            </p>
          {/if}
          {#if document.listType === "documentProcessStage"}
            <p>
              <span class="font-semibold text-primary">Páginas:</span>
              {formatPagesText(document.selectedPages)}
            </p>
          {/if}
          {#if document.listType === "pageProcessStageError"}
            <p class="text-red-500">
              <span class="font-semibold">Erro:</span>
              {document.errorMessage}
            </p>
            <Button
              size="sm"
              variant="outline"
              onclick={() =>
                document.listType === "pageProcessStageError" &&
                handleRetry(document)}
              class="mt-2"
            >
              <RefreshCw class="h-3.5 w-3.5 mr-1" />
              Tentar novamente
            </Button>
          {/if}
          <Collapsible.Root>
            <div class="sticky top-0">
              <Collapsible.Trigger>
                <Button variant="secondary" size="icon" class="h-6 w-6 mt-1">
                  <Ellipsis class="h-3 w-3" />
                </Button>
              </Collapsible.Trigger>
            </div>
            <Collapsible.Content class="mt-1.5 space-y-1">
              <p>
                <span class="font-semibold text-primary">ID:</span>
                {document.id}
              </p>
              <p>
                <span class="font-semibold text-primary">Início:</span>
                {new Date(document.startTime).toLocaleString()}
              </p>
              <p>
                <span class="font-semibold text-primary">Fim:</span>
                {document.listType === "documentProcessStage" &&
                  new Date(document.endTime).toLocaleString()}
              </p>
              {#if document.listType === "pageProcessStageError"}
                <p class="text-red-500">
                  <span class="font-semibold">Erro:</span>
                  {document.errorMessage}
                </p>
              {/if}
              <p class="font-semibold text-primary">info:</p>
              <pre
                class="text-wrap w-full max-w-full overflow-x-auto whitespace-pre-wrap break-words text-[10px]">
                {JSON.stringify(document, null, 2)}
              </pre>
            </Collapsible.Content>
          </Collapsible.Root>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
</div>
