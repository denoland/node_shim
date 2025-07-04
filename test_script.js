console.log("Hello from script!", [...process.argv]);
console.log("Deno" in globalThis ? "Actually running in Deno!" : "Not running in Deno.");