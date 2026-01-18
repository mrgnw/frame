<script lang="ts">
    import { v4 as uuidv4 } from "uuid";
    import { Plus, Play, FileVideo, HardDrive } from "lucide-svelte";

    import FileItemRow from "$lib/components/FileItemRow.svelte";
    import SettingsPanel from "$lib/components/SettingsPanel.svelte";
    import {
        type FileItem,
        FileStatus,
        type ConversionConfig,
    } from "$lib/types";

    const DEFAULT_CONFIG: ConversionConfig = {
        container: "mp4",
        videoCodec: "libx264",
        audioCodec: "aac",
        resolution: "original",
        crf: 23,
        preset: "medium",
    };

    let files = $state<FileItem[]>([]);
    let selectedFileId = $state<string | null>(null);
    let isProcessing = $state(false);

    let selectedFile = $derived(files.find((f) => f.id === selectedFileId));
    let totalSize = $derived(files.reduce((acc, curr) => acc + curr.size, 0));

    function formatTotalSize(bytes: number) {
        if (bytes === 0) return "0 KB";
        const mb = bytes / (1024 * 1024);
        return mb > 1000
            ? `${(mb / 1024).toFixed(2)} GB`
            : `${mb.toFixed(1)} MB`;
    }

    function handleAddFile(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files) {
            const newFiles: FileItem[] = Array.from(target.files).map(
                (f: File) => ({
                    id: uuidv4(),
                    name: f.name,
                    size: f.size,
                    status: FileStatus.IDLE,
                    progress: 0,
                    originalFormat: f.name.split(".").pop() || "unknown",
                    config: { ...DEFAULT_CONFIG },
                    path: `/mock/path/to/${f.name}`,
                }),
            );
            files = [...files, ...newFiles];
            if (!selectedFileId && newFiles.length > 0) {
                selectedFileId = newFiles[0].id;
            }
            target.value = "";
        }
    }

    function handleRemoveFile(id: string) {
        files = files.filter((f) => f.id !== id);
        if (selectedFileId === id) selectedFileId = null;
    }

    function updateSelectedConfig(newConfig: Partial<ConversionConfig>) {
        if (selectedFileId) {
            files = files.map((f) =>
                f.id === selectedFileId
                    ? { ...f, config: { ...f.config, ...newConfig } }
                    : f,
            );
        }
    }

    function startConversion() {
        isProcessing = true;
        files = files.map((f) =>
            f.status === FileStatus.IDLE
                ? { ...f, status: FileStatus.CONVERTING, progress: 0 }
                : f,
        );
    }

    $effect(() => {
        if (!isProcessing) return;

        const interval = setInterval(() => {
            let allDone = true;
            const nextState = files.map((f) => {
                if (f.status === FileStatus.CONVERTING) {
                    const increment = Math.random() * 8 + 2;
                    const newProgress = Math.min(f.progress + increment, 100);
                    if (newProgress < 100) {
                        allDone = false;
                        return { ...f, progress: newProgress };
                    } else {
                        return {
                            ...f,
                            progress: 100,
                            status: FileStatus.COMPLETED,
                        };
                    }
                }
                return f;
            });

            files = nextState;

            if (allDone) isProcessing = false;
        }, 200);

        return () => clearInterval(interval);
    });
</script>

<div
    class="flex flex-col absolute inset-0 bg-black text-foreground font-sans overflow-hidden selection:bg-ds-blue-900 selection:text-white"
>
    <div class="flex-1 p-4 overflow-hidden">
        <div class="grid grid-cols-12 grid-rows-[auto_1fr] gap-4 h-full">
            <div
                class="col-span-12 h-16 border border-ds-gray-100 rounded-lg flex items-center justify-between px-6 shadow-sm"
            >
                <div class="flex items-center gap-6">
                    <div class="flex items-center gap-2">
                        <div
                            class="w-8 h-8 bg-foreground rounded flex items-center justify-center text-black"
                        ></div>
                        <div class="flex flex-col">
                            <span class="text-sm font-bold tracking-tight">
                                Relay
                            </span>
                            <span
                                class="text-[10px] font-mono text-ds-gray-500"
                            >
                                FFMPEG Converter
                            </span>
                        </div>
                    </div>
                    <div class="h-8 w-px bg-ds-gray-100"></div>
                    <div
                        class="flex items-center gap-4 text-xs font-mono text-ds-gray-500"
                    >
                        <div class="flex items-center gap-2">
                            <HardDrive size={14} />
                            <span>STORAGE: {formatTotalSize(totalSize)}</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <FileVideo size={14} />
                            <span>ITEMS: {files.length}</span>
                        </div>
                    </div>
                </div>

                <div class="flex items-center gap-3">
                    <div class="relative group">
                        <input
                            type="file"
                            id="file-upload"
                            multiple
                            class="hidden"
                            onchange={handleAddFile}
                        />
                        <label
                            for="file-upload"
                            class="flex items-center gap-2 bg-ds-gray-100 hover:bg-ds-gray-200 text-foreground px-4 py-2 rounded text-xs font-mono font-medium transition-colors cursor-pointer border border-ds-gray-200 uppercase tracking-wide"
                        >
                            <Plus size={14} />
                            Add Source
                        </label>
                    </div>

                    <button
                        onclick={startConversion}
                        disabled={isProcessing || files.length === 0}
                        class="flex items-center gap-2 px-5 py-2 rounded text-xs font-mono font-medium uppercase tracking-wide transition-all
                    {isProcessing || files.length === 0
                            ? 'bg-black border border-ds-gray-200 text-ds-gray-600 cursor-not-allowed'
                            : 'bg-foreground text-black hover:bg-white border border-foreground'}"
                    >
                        {#if isProcessing}
                            <span class="animate-pulse">PROCESSING...</span>
                        {:else}
                            <Play size={14} fill="currentColor" />
                            START BATCH
                        {/if}
                    </button>
                </div>
            </div>

            <div
                class="col-span-12 lg:col-span-8 border border-ds-gray-100 rounded-lg overflow-hidden flex flex-col relative group"
            >
                <div
                    class="h-10 border-b border-ds-gray-100 backdrop-blur-sm flex items-center px-4 z-10"
                >
                    <div class="w-2.5 mr-4"></div>
                    <div
                        class="flex-1 grid grid-cols-12 gap-4 text-[10px] font-mono text-ds-gray-500 uppercase tracking-widest"
                    >
                        <div class="col-span-5">Name</div>
                        <div class="col-span-3 text-right">Size</div>
                        <div class="col-span-2 text-right">Target</div>
                        <div class="col-span-2 text-right">State</div>
                    </div>
                    <div class="w-8 ml-4"></div>
                </div>

                <div class="flex-1 overflow-y-auto z-10 relative">
                    {#if files.length === 0}
                        <div
                            class="h-full flex flex-col items-center justify-center p-12"
                        >
                            <div class="text-center space-y-2 select-none">
                                <div
                                    class="font-mono text-8xl md:text-9xl text-ds-gray-100 tracking-tighter leading-none"
                                >
                                    00
                                </div>
                                <div
                                    class="text-xl md:text-2xl font-mono text-ds-gray-600 tracking-tight"
                                >
                                    FILES QUEUED
                                </div>
                            </div>
                        </div>
                    {:else}
                        <div>
                            {#each files as file (file.id)}
                                <FileItemRow
                                    item={file}
                                    isSelected={selectedFileId === file.id}
                                    onSelect={(id) => (selectedFileId = id)}
                                    onRemove={handleRemoveFile}
                                />
                            {/each}
                            <div
                                class="p-4 text-center border-t border-ds-gray-100 mt-2"
                            >
                                <span
                                    class="text-[10px] font-mono text-ds-gray-600 uppercase tracking-widest"
                                >
                                    END OF LIST // {files.length} OBJECTS
                                </span>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>

            <div
                class="col-span-12 lg:col-span-4 border border-ds-gray-100 rounded-lg overflow-hidden flex flex-col"
            >
                {#if selectedFile}
                    <SettingsPanel
                        config={selectedFile.config}
                        onUpdate={updateSelectedConfig}
                        disabled={selectedFile.status ===
                            FileStatus.CONVERTING ||
                            selectedFile.status === FileStatus.COMPLETED}
                    />
                {:else}
                    <div
                        class="h-full flex flex-col items-center justify-center text-center p-8 bg-ds-gray-100/5 relative overflow-hidden"
                    >
                        <h3
                            class="text-xs font-mono font-bold uppercase tracking-widest text-ds-gray-400 mb-2"
                        >
                            Awaiting Selection
                        </h3>
                        <p
                            class="text-[10px] font-mono text-ds-gray-600 max-w-50"
                        >
                            SELECT AN ITEM FROM THE QUEUE TO ACCESS
                            CONFIGURATION
                        </p>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>
