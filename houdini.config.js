/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
    "watchSchema": {
        "url": "http://localhost:8000/"
    },
    "runtimeDir": ".houdini",
    "plugins": {
        "houdini-svelte": {
            forceRunesMode: true
        }
    }
}

export default config
