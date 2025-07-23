import * as Comlink from 'comlink';

import * as gltf_model from './gltf-model';

function get_model_array_buffer(model: string): ArrayBuffer {
    let array = new TextEncoder().encode(model);
    return array.buffer.slice(array.byteOffset, array.byteLength + array.byteOffset)
}

export class ParallelWorker {
    private ctx: any;

    async init(canvas: OffscreenCanvas, width: number, height: number) {
        const rayca = await import('./pkg');
        const {} = await rayca.default();
        await rayca.initThreadPool(navigator.hardwareConcurrency);
        let buffer = get_model_array_buffer(gltf_model.EMBEDDED_BOX);
        this.ctx = await rayca.Context.new(canvas, width, height, buffer);
    }

    beginLoop() {
        // Main loop on the worker thread. Execute until `free` is called to set `this.freeFlag`.
        const step = () => {
            this.ctx.draw();
            requestAnimationFrame(step);
        }
        requestAnimationFrame(step);
    }
}

Comlink.expose(ParallelWorker);
