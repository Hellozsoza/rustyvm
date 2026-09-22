/* VirtualBox Browser JavaScript Glue */

class VBoxAPI {
    constructor() {
        this.vmManager = null;
        this.console = null;
        this.network = null;
        this.audio = null;
        this.storage = null;
        this.initialized = false;
    }

    async init() {
        try {
            this.vmManager = new VBoxVMManager();
            this.network = new VBoxNetwork();
            this.audio = new VBoxAudio();
            this.storage = new VBoxStorage();
            this.initialized = true;
            console.log('[VirtualBox] API initialized');
        } catch (e) {
            console.error('[VirtualBox] Init failed:', e);
        }
    }

    createVM(name) {
        return this.vmManager.createVM(name);
    }

    startVM(uuid) {
        return this.vmManager.startVM(uuid);
    }

    stopVM(uuid) {
        return this.vmManager.stopVM(uuid);
    }

    pauseVM(uuid) {
        return this.vmManager.pauseVM(uuid);
    }

    saveState(uuid) {
        return this.vmManager.saveState(uuid);
    }

    loadState(uuid, data) {
        return this.vmManager.loadState(uuid, data);
    }

    captureFrame() {
        return this.console.captureFrame();
    }

    sendKey(code) {
        return this.console.sendKey(code);
    }

    sendMouse(x, y, buttons) {
        return this.console.sendMouse(x, y, buttons);
    }
}

class VBoxVMManager {
    constructor() {
        this.vms = new Map();
        this.currentVM = null;
    }

    createVM(name) {
        const uuid = crypto.randomUUID();
        this.vms.set(uuid, { name, state: 'stopped', progress: 0 });
        this.currentVM = uuid;
        return uuid;
    }

    startVM(uuid) {
        const vm = this.vms.get(uuid);
        if (!vm) throw new Error(`VM ${uuid} not found`);
        vm.state = 'running';
        return true;
    }

    stopVM(uuid) {
        const vm = this.vms.get(uuid);
        if (!vm) throw new Error(`VM ${uuid} not found`);
        vm.state = 'stopped';
        return true;
    }

    pauseVM(uuid) {
        const vm = this.vms.get(uuid);
        if (!vm) throw new Error(`VM ${uuid} not found`);
        vm.state = 'paused';
        return true;
    }

    saveState(uuid) {
        const vm = this.vms.get(uuid);
        if (!vm) throw new Error(`VM ${uuid} not found`);
        return new Uint8Array(0);
    }

    loadState(uuid, data) {
        const vm = this.vms.get(uuid);
        if (!vm) throw new Error(`VM ${uuid} not found`);
        vm.state = 'loading';
        return true;
    }
}

class VBoxConsole {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.width = canvas.width;
        this.height = canvas.height;
        this.frameBuffer = new Uint8Array(this.width * this.height * 4);
        this.running = false;
    }

    captureFrame() {
        const imageData = this.ctx.getImageData(0, 0, this.width, this.height);
        return new Uint8Array(imageData.data);
    }

    sendKey(code) {
        const event = new KeyboardEvent('keydown', { keyCode: code });
        this.canvas.dispatchEvent(event);
    }

    sendMouse(x, y, buttons) {
        const event = new MouseEvent('mousedown', { clientX: x, clientY: y, buttons });
        this.canvas.dispatchEvent(event);
    }

    render(frameData, width, height) {
        const imageData = new ImageData(
            new Uint8ClampedArray(frameData),
            width,
            height
        );
        this.ctx.putImageData(imageData, 0, 0);
    }
}

class VBoxNetwork {
    constructor() {
        this.socket = null;
        this.connected = false;
    }

    async connect(url) {
        this.socket = new WebSocket(url);
        return new Promise((resolve, reject) => {
            this.socket.onopen = () => { this.connected = true; resolve(); };
            this.socket.onerror = reject;
        });
    }

    send(data) {
        if (this.socket && this.connected) {
            this.socket.send(data);
        }
    }

    onMessage(callback) {
        this.socket.onmessage = (e) => callback(e.data);
    }
}

class VBoxAudio {
    constructor() {
        this.context = null;
        this.playing = false;
    }

    async init() {
        this.context = new AudioContext();
    }

    async play(samples) {
        const buffer = this.context.createBuffer(1, samples.length, 44100);
        const channelData = buffer.getChannelData(0);
        channelData.set(samples);
        const source = this.context.createBufferSource();
        source.buffer = buffer;
        source.connect(this.context.destination);
        source.start();
        this.playing = true;
    }

    stop() {
        if (this.context) {
            this.context.close();
        }
        this.playing = false;
    }

    setVolume(volume) {
        this.volume = volume;
    }
}

class VBoxStorage {
    constructor() {
        this.db = null;
    }

    async init() {
        this.db = await idb.openDB('vbox-storage', 1, {
            upgrade(db) {
                db.createObjectStore('vm-states');
                db.createObjectStore('disk-images');
            }
        });
    }

    async saveVMState(uuid, data) {
        await this.db.put('vm-states', data, uuid);
    }

    async loadVMState(uuid) {
        return await this.db.get('vm-states', uuid);
    }

    async saveDiskImage(data, name) {
        await this.db.put('disk-images', data, name);
    }
}

window.vbox = new VBoxAPI();
window.VBoxAPI = VBoxAPI;
window.VBoxVMManager = VBoxVMManager;
window.VBoxConsole = VBoxConsole;
window.VBoxNetwork = VBoxNetwork;
window.VBoxAudio = VBoxAudio;
window.VBoxStorage = VBoxStorage;
