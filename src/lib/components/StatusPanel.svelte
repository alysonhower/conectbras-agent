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
      | DocumentProcessStageModel
      | FinishedDocumentProcessStageModel,
  ): string => {
    if (document instanceof InProcessInstanceModel) {
      if (document.stage instanceof PagePreprocessStageModel) {
        return `Pré-processando ${document.stage.selectedPages.length > 1 ? `páginas` : `página`}.`;
      } else if (document.stage instanceof PagePreprocessStageErrorModel) {
        return `Após erro ao pré-processar ${document.stage.selectedPages.length > 1 ? `as páginas` : `a página`}, tentando novamente.`;
      } else if (document.stage instanceof DocumentProcessStageModel) {
        return `Pré-processamento concluído com sucesso. Gerando documento.`;
      } else {
        return `Após erro ao gerar o documento, tentando novamente.`;
      }
    } else {
      return `Documento gerado com sucesso.`;
    }
  };

  const getContent = (
    document:
      | InProcessInstanceModel
      | PagePreprocessStageModel
      | DocumentProcessStageModel
      | FinishedDocumentProcessStageModel,
  ): string => {
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
    } else {
      return ``;
    }
  };
</script>

<div class="w-full h-full overflow-y-auto p-4 space-y-4">
  {#each allDocuments as document (document.id)}
    {@const title = `${getTitle(document instanceof InProcessInstanceModel ? document.stage.selectedPages : document.selectedPages)}`}
    <Card.Root class="w-full max-h-[50vh] flex flex-col">
      <Card.Header>
        <Card.Title class="text-lg font-semibold break-all">{title}</Card.Title>
        <Card.Description class="text-sm text-gray-500 break-all">
          {getCardDescription(document)}
        </Card.Description>
      </Card.Header>
      <Card.Content class="text-sm flex-grow overflow-y-auto">
        <div class="prose prose-sm max-w-none break-all">
          {@html getContent(document)}
        </div>
      </Card.Content>
      <Card.Footer></Card.Footer>
    </Card.Root>
  {/each}
</div>
