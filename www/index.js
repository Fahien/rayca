import * as rayca from "rayca";

var ctx = null;

const tick = async () => {
    if (ctx == null) {
        ctx = await rayca.Context.new();
    }
    ctx.draw();
    requestAnimationFrame(tick);
}

requestAnimationFrame(tick);
