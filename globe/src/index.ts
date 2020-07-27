import * as THREE from 'three';
import { Matrix4, Vector3 } from 'three';

interface EventHandlers {
	onMouseDown: HTMLElement['onmousedown'],
	onMouseMove: HTMLElement['onmousemove'],
	onMouseUp: HTMLElement['onmouseup'],
	onWheel: HTMLElement['onwheel'],
}

function getContainer(): HTMLElement {
	return document.querySelector('#application');
}

function getToken(): string {
	// @ts-ignore
	if (MAPBOX_TOKEN) {
		// @ts-ignore
		return MAPBOX_TOKEN;
	} else {
		return getContainer().dataset['token'];
	}
}

function createRenderer(handlers: EventHandlers): THREE.WebGLRenderer {
	const el = getContainer();
	// Window
	const renderer = new THREE.WebGLRenderer({ antialias: true });
	//renderer.setClearColor(0x000511, 1);
	renderer.setSize(window.innerWidth - 20, window.innerHeight - 20);
	el.appendChild(renderer.domElement);



	window.addEventListener('mousedown', handlers.onMouseDown);
	window.addEventListener('mouseup', handlers.onMouseUp);
	window.addEventListener('mousemove', handlers.onMouseMove);
	window.addEventListener('wheel', handlers.onWheel);

	return renderer;
}


// Funky async wasm import
import("../node_modules/papariki/papariki").then(papariki => {
	papariki.setToken(getToken());
	let transform = new Matrix4();
	transform.makeRotationAxis(new Vector3(0, 1, 0), 0.4);

	let mouseDown = false;
	let mousePosition = [0.0, 0.0];
	let rotation = [Math.PI * 0.2, Math.PI];
	let zoom = 1.0;
	//let tilt = -0.41;
	let tilt = 0.0;
	function updateTransform() {
		// Update transform matrix
		transform = new Matrix4()
			.multiply(new Matrix4()
				.makeRotationZ(tilt)
			)
			.multiply(new Matrix4()
				.makeRotationX(rotation[0])
			)
			.multiply(new Matrix4()
				.makeRotationY(rotation[1])
			);
	}
	updateTransform();

	const renderer = createRenderer({
		onWheel(e) {
			let dt = Math.abs(e.deltaY) * 0.1;
			// Hack for chrome
			if (dt > 5 ) {
				dt -= 5;
			}
			let scale = (2.0 - zoom);
			if (e.deltaY < 0) {
				zoom += dt * scale;
			} else {
				zoom -= dt * scale;
			}
			if (zoom > 1.98) {
				zoom = 1.98;
			}
			if (zoom < 0.33) {
				zoom = 0.33;
			}
			camera.position.z = 3 - zoom;
		},
		onMouseDown(e) {
			e.preventDefault();
			mouseDown = true;
			mousePosition = [e.clientX, e.clientY];
		},
		onMouseUp(e) {
			mouseDown = false;
			mousePosition = [e.clientX, e.clientY];
		},
		onMouseMove(e) {
			if (!mouseDown) return;

			const deltaX = e.clientX - mousePosition[0];
			const deltaY = e.clientY - mousePosition[1];
			mousePosition = [e.clientX, e.clientY];

			rotation[0] += 0.005 * deltaY * ((camera.position.z - 1.0) * 0.5);
			rotation[1] += 0.005 * deltaX * ((camera.position.z - 1.0) * 0.5);

			const limit = 0.4;
			if (rotation[0] > Math.PI * limit) {
				rotation[0] = Math.PI * limit;
			}
			if (rotation[0] < -Math.PI * limit) {
				rotation[0] = -Math.PI * limit;
			}
			updateTransform();
		},
	});

	// Camera
	const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.001, 10);
	camera.position.z = 2;

	// Globe
	const scene = new THREE.Scene();
	const meshes: Array<THREE.Mesh> = [];


	// Fetch the geometry
	const tileZoom = 1;
	const n = Math.pow(2, tileZoom);
	const tiles = [];
	setTimeout(async () => {
		for (let y = 0; y < n; y++) {
			for (let x = 0; x < n; x++) {
				setTimeout(async () => {
					const tile = await papariki.getTile(x, y, tileZoom);
					const geometry = new THREE.BufferGeometry();
					const triangles = Array.from(tile.triangles()) as Array<number>;
					const vertices = tile.vertices();
					geometry.setIndex(triangles);
					geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));

					//const material = new THREE.MeshBasicMaterial({ color: 0x000000 });
					const material = new THREE.MeshBasicMaterial({ color: 0x22dd99 });
					const mesh = new THREE.Mesh(geometry, material);
					scene.add(mesh);
					meshes.push(mesh);
				}, 500 * (y * n + x));

			}
		}
	});









	function createOcean() {
		const geometry = new THREE.SphereBufferGeometry(1, 32, 32);
		const material = new THREE.MeshStandardMaterial({color: 0x000000, opacity: 0.7, transparent: true});
		//const material = new THREE.MeshStandardMaterial({color: 0x9ce3f9});
		const sphere = new THREE.Mesh(geometry, material);
		scene.add(sphere);
	}

	// Ocean
	const ocean = createOcean();

	// Lighting
	const light = new THREE.DirectionalLight(0xffffff, 0.5);
	light.position.set(-0.5, 1.2, 1.2);
	scene.add(light);
	scene.add(new THREE.AmbientLight(0x303030));
	//scene.add(new THREE.HemisphereLight(0xffffbb, 0x080820, 1));

	let time = performance.now();
	function update(dt: number) {
		if (!mouseDown) {
			rotation[1] += 0.2 * dt;
		}
		updateTransform();
		// Update rotations
		for (let mesh of meshes) {
			mesh.matrix = transform.clone();
			mesh.matrixAutoUpdate = false;
		}
	}

	function animate() {
		const nextTime = performance.now() / 1000.0;
		const dt = nextTime - time;
		time = nextTime;

		update(dt);

		renderer.render(scene, camera);
		requestAnimationFrame(animate);
	}
	animate();
});
