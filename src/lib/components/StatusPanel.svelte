<!-- <script lang="ts">
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
    type  ExtractDocumentImagesStage,
    type  ExtractDocumentImagesStageSuccess,
    type PagePreprocessStageSuccess,
    type PagePreprocessStage,
    type DocumentProcessStage,
    type DocumentProcessStageError,
    type DocumentProcessStageSuccess,
    type FinishedDocumentProcessStage,
    type PagePreprocessStageError,
    type inProcess,
  } from "./processWorkflowContext.svelte";

  const renderState = globalSetupState.state;

  const allDocuments = $derived.by(() => {
    const inProcessLIst: inProcess[] = renderState.inProcessList;
    const pageProcessStageErrorList: PagePreprocessStageError[] =
      renderState.pageProcessStageErrorList;
    const pageProcessStageSuccessList: PagePreprocessStageSuccess[] =
      renderState.pageProcessStageSuccessList;
    const documentProcessStageErrorList: DocumentProcessStageError[] =
      renderState.documentProcessStageErrorList;
    const documentProcessStageSuccessList: DocumentProcessStageSuccess[] =
      renderState.documentProcessStageSuccessList;
    const finishedDocumentsProcessStage: FinishedDocumentProcessStage[] =
      renderState.finishedDocumentsProcessStage;
    return [
      ...inProcessLIst,
      ...pageProcessStageErrorList,
      ...pageProcessStageSuccessList,
      ...documentProcessStageErrorList,
      ...documentProcessStageSuccessList,
      ...finishedDocumentsProcessStage,
    ].sort((a, b) => {
      const aFirstPage = Math.min(...a.selectedPages);
      const bFirstPage = Math.min(...b.selectedPages);
      return aFirstPage - bFirstPage;
    });
  });
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Card Title</Card.Title>
    <Card.Description>Card Description</Card.Description>
  </Card.Header>
  <Card.Content>
    <p>Card Content</p>
    {#each allDocuments as document}
      document.
    {/each}
  </Card.Content>
  <Card.Footer>
    <p>Card Footer</p>
  </Card.Footer>
</Card.Root> -->
