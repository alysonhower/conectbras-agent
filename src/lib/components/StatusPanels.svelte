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

  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Separator } from "$lib/components/ui/separator";
  import * as Dialog from "$lib/components/ui/dialog";
    import {
    globalSetupState,
    type PagePreprocessStageSuccess,
    type PagePreprocessStageResult,
    type PagePreprocessStage,
    type DocumentProcessStage,
    type FinishedDocumentStage,
  } from "./processWorkflowContext.svelte";

  const renderState = globalSetupState.state;
  const statePaths = globalSetupState.paths;
  
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

  let time = $state(new Date());
  let editingDocumentId = $state<string | undefined>(undefined);
  let editedFileName = $state("");
  let showHistoryMap = $state(new Map<string, boolean>());
  let isDropdownOpenMap = $state(new Map<string, boolean>());
  let historyHoverTimeoutMap = $state(new Map<string, NodeJS.Timeout>());
  let confirmProcessDialogOpenMap = $state(new Map<string, boolean>());

  type AllDocumentTypes =
    | (PagePreprocessStage & { listType: "processing"; info: PagePreprocessStageResult })
    | (DocumentProcessStage & { listType: "processed" })
    | (FinishedDocumentStage & { listType: "finished" });

  const allDocuments = $derived(
    [
      ...renderState.preprocessPagesStage.map((doc) => ({
        ...doc,
        listType: "processing" as const,
        info: {} as PagePreprocessStageResult,
      })),
      ...renderState.finishedDocumentsStage.map((doc) => ({
        ...doc,
        listType: "processed" as const,
      })),
      ...renderState.finishedDocumentsStage.map((doc) => ({
        ...doc,
        listType: "finished" as const,
      })),
    ].sort((a, b) => Math.min(...a.selectedPages) - Math.min(...b.selectedPages)),
  );

  const setMapValue = (map: Map<string, any>, id: string, value: any) => {
    return new Map(map).set(id, value);
  };

  const handleHistoryInteraction = (id: string, isEnter: boolean) => {
    clearTimeout(historyHoverTimeoutMap.get(id));
    if (isEnter) {
      showHistoryMap = setMapValue(showHistoryMap, id, true);
      isDropdownOpenMap = setMapValue(isDropdownOpenMap, id, true);
    } else {
      const timeout = setTimeout(() => {
        showHistoryMap = setMapValue(showHistoryMap, id, false);
        isDropdownOpenMap = setMapValue(isDropdownOpenMap, id, false);
      }, 300);
      historyHoverTimeoutMap = setMapValue(historyHoverTimeoutMap, id, timeout);
    }
  };

  const handleEditButtonInteraction = (id: string, isEnter: boolean) => {
    if (isEnter) {
      showHistoryMap = setMapValue(showHistoryMap, id, true);
    } else {
      const timeout = setTimeout(() => {
        if (!isDropdownOpenMap.get(id)) {
          showHistoryMap = setMapValue(showHistoryMap, id, false);
        }
      }, 300);
      historyHoverTimeoutMap = setMapValue(historyHoverTimeoutMap, id, timeout);
    }
  };

  const startEditing = (
    document:
      | (DocumentProcessStage & { listType: "processed" })
      | (FinishedDocumentStage & { listType: "finished" }),
  ) => {
    editingDocumentId = document.id;
    editedFileName = document.fileName;
  };

  const saveFileName = async (
    document:
      | (DocumentProcessStage & { listType: "processed" })
      | (FinishedDocumentStage & { listType: "finished" }),
  ) => {
    if (!editedFileName.trim()) return;

    try {
      let updatedDocumentInfo: PagePreprocessStageResult;
      if (document.listType === "finished") {
        console.log(
          "Renaming finished document:",
          "json path: " + document.imagesDirectory,
          "new name: " + editedFileName,
        );
        updatedDocumentInfo = await invoke("rename_finished_document", {
          oldPath: document.info.json_file_path,
          newName: editedFileName,
        });
      } else {
        updatedDocumentInfo = await invoke("update_file_name", {
          path: document.json_file_path,
          name: editedFileName,
        });
      }
      console.log("Updated document info:", updatedDocumentInfo);

      if (!updatedDocumentInfo.file_name_history) {
        updatedDocumentInfo.file_name_history = [];
      }
      if (!updatedDocumentInfo.file_name_history.includes(editedFileName)) {
        updatedDocumentInfo.file_name_history.push(editedFileName);
      }

      if (document.listType === "finished") {
        processWorkflowContext.finishedDocumentsStage =
          processWorkflowContext.finishedDocumentsStage.map((doc) =>
            doc.id === document.id
              ? {
                  ...doc,
                  file_name: updatedDocumentInfo.file_name,
                  info: updatedDocumentInfo,
                }
              : doc,
          );
      } else {
        processWorkflowContext.processDocumentsStage =
          processWorkflowContext.processDocumentsStage.map((doc) =>
            doc.id === document.id
              ? {
                  ...doc,
                  file_name: updatedDocumentInfo.file_name,
                  info: updatedDocumentInfo,
                }
              : doc,
          );
      }

      showHistoryMap = setMapValue(showHistoryMap, document.id, false);
    } catch (error) {
      console.error("Error updating file name:", error);
    } finally {
      editingDocumentId = undefined;
    }
  };

  const cancelEditing = () => {
    editingDocumentId = undefined;
    editedFileName = "";
  };

  const handleKeyDown = async (
    event: KeyboardEvent,
    document:
      | (DocumentProcessStage & { listType: "processed" })
      | (FinishedDocumentStage & { listType: "finished" }),
  ) => {
    if (event.key === "Enter") await saveFileName(document);
    else if (event.key === "Escape") cancelEditing();
  };

  const elapsedTimes = $derived(
    processWorkflowContext.preprocessPagesStage.map((page) => ({
      id: page.id,
      elapsed: formatDurationText(page.startTime, time.getTime()),
    })),
  );

  const removeFromProcessed = (document: AllDocumentTypes) => {
    processWorkflowContext.processDocumentsStage =
      processWorkflowContext.processDocumentsStage.filter(
        (doc) => doc.id !== document.id,
      );
  };

  const addToProcessing = (document: AllDocumentTypes) => {
    processWorkflowContext.preprocessPagesStage = [
      ...processWorkflowContext.preprocessPagesStage,
      { ...document, status: "processing", startTime: Date.now() },
    ];
  };

  const isProcessedOrFinished = (
    doc: AllDocumentTypes,
  ): doc is (ProcessDocumentStage | FinishedDocumentStage) & {
    listType: "processed" | "finished";
  } => doc.listType === "processed" || doc.listType === "finished";

  const handleFinalPipeline = async (document: AllDocumentTypes) => {
    if (!isProcessedOrFinished(document)) return;
    setConfirmProcessDialogOpen(document.id, false);

    removeFromProcessed(document);
    addToProcessing(document);

    try {
      await invoke("final_pipeline", { documentInfo: document.info });
      document.status = "completed";
      document.endTime = Date.now();
      processWorkflowContext.finishedDocumentsStage = [
        ...processWorkflowContext.finishedDocumentsStage,
        document,
      ];
    } catch (error) {
      console.error("Error in final pipeline:", error);
      document.status = "error";
      document.error = error instanceof Error ? error.message : String(error);
      processWorkflowContext.processDocumentsStage = [
        ...processWorkflowContext.processDocumentsStage,
        document,
      ];
    } finally {
      processWorkflowContext.preprocessPagesStage = processWorkflowContext.preprocessPagesStage.filter(
        (page) => page.id !== document.id,
      );
    }
  };

  const setConfirmProcessDialogOpen = (id: string, isOpen: boolean) => {
    confirmProcessDialogOpenMap = new Map(confirmProcessDialogOpenMap).set(
      id,
      isOpen,
    );
  };

  const isConfirmProcessDialogOpen = (id: string) => {
    return confirmProcessDialogOpenMap.get(id) || false;
  };

  const handleRetry = async (document: AllDocumentTypes) => {
    if (document.listType !== "processed") return; // Only processed documents can be retried

    const newProcess: PreprocessPagesStage = {
      id: uuidv4(),
      imagesDirectory: document.imagesDirectory,
      pages: [...document.pages],
      status: "processing",
      startTime: Date.now(),
    };

    processWorkflowContext.preprocessPagesStage = [
      ...processWorkflowContext.preprocessPagesStage,
      newProcess,
    ];

    processWorkflowContext.processDocumentsStage =
      processWorkflowContext.processDocumentsStage.filter(
        (doc) => doc.id !== document.id,
      );

    try {
      const res = await invoke<DocumentInfo>("anthropic_pipeline", {
        paths: newProcess.pages,
      });

      const newProcessedDocument: ProcessDocumentStage & {
        listType: "processed";
      } = {
        ...newProcess,
        listType: "processed",
        file_name: res.file_name,
        json_file_path: res.json_file_path,
        info: {
          ...res,
        },
        status: "completed",
        endTime: Date.now(),
      };

      processWorkflowContext.processDocumentsStage = [
        ...processWorkflowContext.processDocumentsStage,
        newProcessedDocument,
      ];
    } catch (err) {
      console.error("Error in anthropic_pipeline:", err);
      const errorProcessedDocument: ProcessDocumentStage & {
        listType: "processed";
      } = {
        ...newProcess,
        listType: "processed",
        file_name: "",
        json_file_path: "",
        info: {} as DocumentInfo,
        status: "error",
        endTime: Date.now(),
        error: err instanceof Error ? err.toString() : String(err),
      };
      processWorkflowContext.processDocumentsStage = [
        ...processWorkflowContext.processDocumentsStage,
        errorProcessedDocument,
      ];
    } finally {
      processWorkflowContext.preprocessPagesStage = processWorkflowContext.preprocessPagesStage.filter(
        (pp) => pp.id !== newProcess.id,
      );
    }
  };

  const isCurrentPage = (document: AllDocumentTypes) => {
    return document.pages.includes(processWorkflowContext.currentPageNumber);
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

  const openInExplorer = async (document: FinishedDocumentStage) => {
    try {
      const fullPath =
        document.info.json_file_path
          .replace(/(.+)-data(\\|\/)[^\\\/]+\.json$/, "$1-data\\done\\")
          .replace(/\//g, "\\") +
        document.file_name +
        ".pdf";
      console.log(fullPath);
      await invoke("open_in_explorer", { path: fullPath });
    } catch (error) {
      console.error("Error opening file in explorer:", error);
    }
  };

  const getRingStyle = (document: AllDocumentTypes): string => {
    if (document.listType === "finished") {
      return "ring-4 ring-[rgba(0,128,0,1)]";
    } else if (document.listType === "processed") {
      return "ring-4 ring-[rgba(186,79,125,1)]";
    } else {
      return "ring-4 ring-[rgba(255,165,0,0.8)]";
    }
  }

  $effect(() => {
    const interval = setInterval(() => (time = new Date()), 1000);
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
              {#if document.listType === "processing"}
                Processando ({document.pages.length > 1 ? "páginas" : "página"}: {formatPagesText(
                  document.pages,
                )})
              {:else if document.listType === "finished"}
                Documento Finalizado:
              {:else}
                Nome sugerido (página: {formatPagesText(document.pages)}):
              {/if}
            </span>
            {#if document.listType !== "processing"}
              {#if editingDocumentId === document.id}
                <div class="w-full h-full space-y-1">
                  <!-- svelte-ignore a11y_autofocus -->
                  <div
                    class="w-full p-2 border rounded-md font-semibold text-sm focus:outline-none focus:ring-1 focus:ring-primary break-all min-h-[1.5em] max-h-[50vh] overflow-y-auto"
                    contenteditable="true"
                    bind:textContent={editedFileName}
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
                      disabled={!editedFileName.trim()}
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
                  {document.file_name}
                </span>
                <div class="flex justify-end space-x-1 relative">
                  <div class="relative">
                    {#if document.info?.file_name_history && document.info.file_name_history.length > 1}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="absolute right-full transition-all duration-300 ease-in-out transform"
                        class:translate-x-[100%]={!showHistoryMap.get(
                          document.id,
                        )}
                        class:opacity-0={!showHistoryMap.get(document.id)}
                        class:pointer-events-none={!showHistoryMap.get(
                          document.id,
                        )}
                        onmouseenter={() =>
                          handleHistoryInteraction(document.id, true)}
                        onmouseleave={() =>
                          handleHistoryInteraction(document.id, false)}
                      >
                        <DropdownMenu.Root
                          open={isDropdownOpenMap.get(document.id)}
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
                          {#key document.info.file_name_history}
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
                                {#each document.info.file_name_history
                                  .slice()
                                  .reverse() as historyItem}
                                  <DropdownMenu.Item
                                    onclick={() => {
                                      editedFileName = historyItem;
                                      saveFileName(document);
                                      isDropdownOpenMap = setMapValue(
                                        isDropdownOpenMap,
                                        document.id,
                                        false,
                                      );
                                    }}
                                    class="break-all cursor-pointer"
                                  >
                                    {historyItem}
                                    {historyItem === document.file_name
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
                    {#if document.listType === "finished"}
                      <Button
                        size="icon"
                        variant="default"
                        onclick={() => openInExplorer(document)}
                        class="h-7 w-7"
                      >
                        <FolderOpen class="h-3.5 w-3.5" />
                      </Button>
                    {:else if document.listType === "processed"}
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
                              {document.pages.length > 1
                                ? "Processar as páginas selecionadas?"
                                : "Processar página selecionada?"}
                            </Dialog.Title>
                            <Dialog.Description>
                              Você está prestes a processar {document.pages
                                .length > 1
                                ? "as páginas"
                                : "a página"}
                              {formatPagesText(document.pages)}. Deseja
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
            {document.listType === "finished"
              ? "Finalizado"
              : translateStatusText(document.status)}
          </p>
          {#if document.listType === "processing"}
            <p>
              <span class="font-semibold text-primary">Tempo decorrido:</span>
              {#key time}<span
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
          {#if document.listType === "finished"}
            <p>
              <span class="font-semibold text-primary">Páginas:</span>
              {formatPagesText(document.pages)}
            </p>
          {/if}
          {#if document.status === "error"}
            <p class="text-red-500">
              <span class="font-semibold">Erro:</span>
              {document.error}
            </p>
            <Button
              size="sm"
              variant="outline"
              onclick={() =>
                document.listType === "processed" && handleRetry(document)}
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
                {new Date(document.endTime!).toLocaleString()}
              </p>
              {#if document.error}
                <p class="text-red-500">
                  <span class="font-semibold">Erro:</span>
                  {document.error}
                </p>
              {/if}
              <p class="font-semibold text-primary">info:</p>
              <pre
                class="text-wrap w-full max-w-full overflow-x-auto whitespace-pre-wrap break-words text-[10px]">
                {JSON.stringify(document.info, null, 2)}
              </pre>
            </Collapsible.Content>
          </Collapsible.Root>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
</div>
