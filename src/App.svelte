<script>
  import "./app.css";
  import { ModeWatcher } from "mode-watcher";
  import ResizableLayout from "$lib/components/ResizableLayout.svelte";
  import LightSwitch from "$lib/components/LightSwitch.svelte";
</script>

<main class="h-screen w-screen">
  <ModeWatcher />
  <div class="absolute left-2 bottom-2">
    <LightSwitch />
  </div>
  <ResizableLayout />
</main>
