
export type CropRect = { x: number; y: number; width: number; height: number };
export type DragHandle = 'move' | 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';

export const MIN_CROP = 0.05;

export const ASPECT_OPTIONS = [
	{ id: 'free', labelKey: 'crop.free', display: 'Free', ratio: null },
	{ id: '1:1', labelKey: null, display: '1:1', ratio: 1 },
	{ id: '4:5', labelKey: null, display: '4:5', ratio: 4 / 5 },
	{ id: '16:9', labelKey: null, display: '16:9', ratio: 16 / 9 },
	{ id: '9:16', labelKey: null, display: '9:16', ratio: 9 / 16 }
];

export function getAspectValue(id: string): number | null {
	const option = ASPECT_OPTIONS.find((opt) => opt.id === id);
	return option?.ratio ?? null;
}

export function clamp(value: number, min: number, max: number): number {
	return Math.min(Math.max(value, min), max);
}

export function clampRect(rect: CropRect): CropRect {
	let { x, y, width, height } = rect;
	if (width < MIN_CROP) width = MIN_CROP;
	if (height < MIN_CROP) height = MIN_CROP;
	if (x < 0) x = 0;
	if (y < 0) y = 0;
	if (x + width > 1) x = 1 - width;
	if (y + height > 1) y = 1 - height;
	return { x, y, width, height };
}

export function getEffectiveAspectRatio(
	targetRatio: number,
	width: number,
	height: number,
	isSideRotation: boolean
): number {
	if (!width || !height) return targetRatio;
	const physicalAspect = width / height;

	if (isSideRotation) {
		return 1 / targetRatio / physicalAspect;
	}
	return targetRatio / physicalAspect;
}

export function transformCropRect(
	rect: CropRect,
	rot: string,
	fH: boolean,
	fV: boolean,
	inverse: boolean
): CropRect {
	let cx = rect.x + rect.width / 2 - 0.5;
	let cy = rect.y + rect.height / 2 - 0.5;
	let w = rect.width;
	let h = rect.height;

	const rotate = () => {
		if (rot === '90') {
			[cx, cy] = [-cy, cx];
			[w, h] = [h, w];
		} else if (rot === '180') {
			[cx, cy] = [-cx, -cy];
		} else if (rot === '270') {
			[cx, cy] = [cy, -cx];
			[w, h] = [h, w];
		}
	};

	const invRotate = () => {
		if (rot === '90') {
			[cx, cy] = [cy, -cx];
			[w, h] = [h, w];
		} else if (rot === '180') {
			[cx, cy] = [-cx, -cy];
		} else if (rot === '270') {
			[cx, cy] = [-cy, cx];
			[w, h] = [h, w];
		}
	};

	const flip = () => {
		if (fH) cx = -cx;
		if (fV) cy = -cy;
	};

	if (inverse) {
		flip();
		invRotate();
	} else {
		rotate();
		flip();
	}

	return {
		x: cx - w / 2 + 0.5,
		y: cy - h / 2 + 0.5,
		width: w,
		height: h
	};
}

export function remapDragDeltas(
	dx: number,
	dy: number,
	rotation: string,
	flipHorizontal: boolean,
	flipVertical: boolean
): { dx: number; dy: number } {
	let rDx = dx;
	let rDy = dy;

	switch (rotation) {
		case '90':
			rDx = dy;
			rDy = -dx;
			break;
		case '180':
			rDx = -dx;
			rDy = -dy;
			break;
		case '270':
			rDx = -dy;
			rDy = dx;
			break;
	}

	if (flipHorizontal) rDx = -rDx;
	if (flipVertical) rDy = -rDy;

	return { dx: rDx, dy: rDy };
}

export function adjustRectToRatio(
	rect: CropRect,
	ratio: number,
	width: number,
	height: number,
	isSideRotation: boolean
): CropRect {
	const effectiveRatio = getEffectiveAspectRatio(ratio, width, height, isSideRotation);

	let w = rect.width;
	let h = rect.height;
	if (w / h > effectiveRatio) {
		w = h * effectiveRatio;
	} else {
		h = w / effectiveRatio;
	}
	const centerX = rect.x + rect.width / 2;
	const centerY = rect.y + rect.height / 2;
	let nextX = centerX - w / 2;
	let nextY = centerY - h / 2;
	if (nextX < 0) nextX = 0;
	if (nextY < 0) nextY = 0;
	if (nextX + w > 1) nextX = 1 - w;
	if (nextY + h > 1) nextY = 1 - h;
	return { x: nextX, y: nextY, width: w, height: h };
}

export function enforceAspect(
	rect: CropRect,
	handle: DragHandle,
	startRect: CropRect,
	ratio: number,
	sourceWidth: number,
	sourceHeight: number,
	isSideRotation: boolean
): CropRect {
	const effectiveRatio = getEffectiveAspectRatio(
		ratio,
		sourceWidth,
		sourceHeight,
		isSideRotation
	);

	let width = rect.width;
	let height = rect.height;
	if (width / height > effectiveRatio) {
		width = height * effectiveRatio;
	} else {
		height = width / effectiveRatio;
	}

	const next = { ...rect };
	switch (handle) {
		case 'e':
			next.x = startRect.x;
			next.width = width;
			{
				const centerY = startRect.y + startRect.height / 2;
				next.y = centerY - height / 2;
				next.height = height;
			}
			break;
		case 'w':
			next.width = width;
			next.x = startRect.x + startRect.width - width;
			{
				const centerY = startRect.y + startRect.height / 2;
				next.y = centerY - height / 2;
				next.height = height;
			}
			break;
		case 'n':
			next.height = height;
			next.y = startRect.y + startRect.height - height;
			{
				const centerX = startRect.x + startRect.width / 2;
				next.x = centerX - width / 2;
				next.width = width;
			}
			break;
		case 's':
			next.height = height;
			next.y = startRect.y;
			{
				const centerX = startRect.x + startRect.width / 2;
				next.x = centerX - width / 2;
				next.width = width;
			}
			break;
		case 'ne':
			next.x = startRect.x;
			next.y = startRect.y + startRect.height - height;
			next.width = width;
			next.height = height;
			break;
		case 'nw':
			next.width = width;
			next.height = height;
			next.x = startRect.x + startRect.width - width;
			next.y = startRect.y + startRect.height - height;
			break;
		case 'se':
			next.x = startRect.x;
			next.y = startRect.y;
			next.width = width;
			next.height = height;
			break;
		case 'sw':
			next.width = width;
			next.height = height;
			next.x = startRect.x + startRect.width - width;
			next.y = startRect.y;
			break;
		default:
			break;
	}

	return next;
}

export function getHandleCursor(handleId: string, isSideRotation: boolean): string {
	if (handleId === 'n' || handleId === 's') return isSideRotation ? 'ew-resize' : 'ns-resize';
	if (handleId === 'e' || handleId === 'w') return isSideRotation ? 'ns-resize' : 'ew-resize';
	if (handleId === 'nw' || handleId === 'se')
		return isSideRotation ? 'nesw-resize' : 'nwse-resize';
	if (handleId === 'ne' || handleId === 'sw')
		return isSideRotation ? 'nwse-resize' : 'nesw-resize';
	return 'default';
}
