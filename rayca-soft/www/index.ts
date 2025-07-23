import * as Comlink from 'comlink';
import { ParallelWorker } from './worker';

import('./pkg').then(async rayca => {
    const { } = await rayca.default();

    const RemoteParallelWorker = Comlink.wrap<typeof ParallelWorker>(
        new Worker(new URL('./worker', import.meta.url), {
            type: 'module'
        })
    );
    let worker = await new RemoteParallelWorker();
    const $ = (id: string) => document.getElementById(id);
    let canvas = $('area') as HTMLCanvasElement;
    let width = window.innerWidth;
    let height = window.innerHeight;
    let offscreen = canvas.transferControlToOffscreen();
    await worker.init(
        Comlink.transfer(offscreen, [offscreen]),
        width,
        height
    );
    await worker.beginLoop();
}).catch(console.error);
