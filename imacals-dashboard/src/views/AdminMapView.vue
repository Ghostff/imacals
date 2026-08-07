<script setup lang="ts">
import { onMounted, onUnmounted, ref, type Ref } from 'vue';
import { setOptions, importLibrary } from '@googlemaps/js-api-loader';
import LocationPickerModal from '@/components/LocationPickerModal.vue';
import { polygonService, type PolygonCoord, type SavedPolygon } from '@/services/polygon';
import { neighborService, type PolygonNeighbor } from '@/services/neighbor';
import { polygonZoneService, type PolygonZone } from '@/services/polygonZone';

const ZONE_COLORS: string[] = [
  '#e83015', '#ff9900', '#efbb24', '#81cb1c', '#56a658', '#33a6b8',
  '#a7c4e7', '#113285', '#8b81c3', '#6f3381', '#cb1b45', '#b8921d',
  '#bec23f', '#f2cd9f', '#eea9a9', '#fedfe1', '#b7b7b7', '#353f4e',
  '#0c0c0c', '#ffffff',
];

const mapEl: Ref<HTMLDivElement | null> = ref(null);
const showModal: Ref<boolean> = ref(true);
const undoVisible: Ref<boolean> = ref(false);

// Polygon zone panel reactive state.
const polygonZoneMode: Ref<boolean>             = ref(false);
const polygonZones: Ref<PolygonZone[]>          = ref([]);
const selectedPolygonZone: Ref<PolygonZone | null> = ref(null);
const newPolygonZoneName: Ref<string>      = ref('');
const newPolygonZoneColor: Ref<string>     = ref(ZONE_COLORS[0]);
const showCreatePolygonZone: Ref<boolean>  = ref(false);
const creatingPolygonZone: Ref<boolean>    = ref(false);

const editingZoneId: Ref<string | null>    = ref(null);
const editingZoneName: Ref<string>         = ref('');
const editingZoneColor: Ref<string>        = ref(ZONE_COLORS[0]);
const savingZone: Ref<boolean>             = ref(false);

let map: google.maps.Map | null = null;
let drawingManager: google.maps.drawing.DrawingManager | null = null;
let activeCityId: string | undefined = undefined;

let deleteMode: boolean = false;
let editMode: boolean   = false;
let linkMode: boolean   = false;

let drawBtn: HTMLButtonElement | null        = null;
let editBtn: HTMLButtonElement | null        = null;
let deleteBtn: HTMLButtonElement | null      = null;
let linkBtn: HTMLButtonElement | null        = null;
let polygonZoneBtn: HTMLButtonElement | null = null;

// The polygon currently open for editing (one at a time).
let editingOverlay: { overlay: google.maps.Polygon; id: string } | null = null;

// State for the rubber-band link line (follows cursor after first polygon is clicked).
let linkSource: { overlay: google.maps.Polygon; polygon: SavedPolygon } | null = null;
let rubberband: google.maps.Polyline | null = null;
let mousemoveListener: google.maps.MapsEventListener | null = null;
let mapClickListener: google.maps.MapsEventListener | null  = null;
// Per-overlay mousemove listeners — polygons swallow map-level events, so we mirror
// the same handler onto every overlay to keep the rubberband tracking the cursor.
let overlayMousemoveListeners: google.maps.MapsEventListener[] = [];

// Lookup table so neighbor rendering can find a polygon's coordinates by id.
const polygonMap = new Map<string, SavedPolygon>();

// Tracks rendered overlays by polygon ID so polygon zone colors can be applied/removed.
const overlayMap = new Map<string, google.maps.Polygon>();

// Tracks rendered neighbor lines keyed by canonical pair (smaller_id:larger_id).
const neighborLines = new Map<string, google.maps.Polyline>();

let undoPending: { coords: PolygonCoord[]; cityId: string | null } | null = null;
let undoTimer: ReturnType<typeof setTimeout> | null = null;

const FILL_OPACITY_DEFAULT: number = 0.15;
const FILL_OPACITY_ZONED: number   = 0.5;

const apiKey = import.meta.env.VITE_GOOGLE_API_KEY as string;
const mapId  = import.meta.env.VITE_GOOGLE_DRAWING_MAP_ID as string;


onMounted(async (): Promise<void> => {
  if (!mapEl.value) return;

  setOptions({ key: apiKey, v: 'weekly' });

  const { Map } = await importLibrary('maps');
  const { DrawingManager, OverlayType } = await importLibrary('drawing');

  map = new Map(mapEl.value, {
    center: { lat: 25.7617, lng: -80.1918 },
    zoom: 12,
    mapId,
    disableDefaultUI: false,
    mapTypeControl: false,
  });

  drawingManager = new DrawingManager({
    drawingMode: null,
    drawingControl: false,
    polygonOptions: {
      fillColor: '#B8422E',
      fillOpacity: 0.15,
      strokeColor: '#B8422E',
      strokeWeight: 2,
    },
  });

  drawingManager.setMap(map);
  await loadFontAwesome();
  addControlGroup(map, OverlayType);

  // Load polygons and zones in parallel so zone colors can be applied immediately.
  const [saved, loadedZones] = await Promise.all([
    polygonService.index().catch((): SavedPolygon[] => []),
    polygonZoneService.index().catch((): PolygonZone[] => []),
  ]);
  polygonZones.value = loadedZones;

  for (const p of saved) {
    polygonMap.set(p.id, p);
    const zone  = loadedZones.find((z: PolygonZone) => z.id === p.polygon_zone_id);
    const color = zone?.color ?? '#B8422E';
    const overlay = new google.maps.Polygon({
      paths: p.coordinates,
      fillColor: color,
      fillOpacity: zone ? FILL_OPACITY_ZONED : FILL_OPACITY_DEFAULT,
      strokeColor: color,
      strokeWeight: 2,
      map,
    });
    overlayMap.set(p.id, overlay);
    attachHandlers(overlay, p);
  }

  const neighbors: PolygonNeighbor[] = await neighborService.index().catch((): PolygonNeighbor[] => []);
  for (const n of neighbors) {
    const a = polygonMap.get(n.polygon_id);
    const b = polygonMap.get(n.neighbor_polygon_id);
    if (a && b) renderNeighborLine(a, b);
  }

  google.maps.event.addListener(drawingManager, 'overlaycomplete', async (e: google.maps.drawing.OverlayCompleteEvent): Promise<void> => {
    if (e.type !== OverlayType.POLYGON) return;

    const overlay = e.overlay as google.maps.Polygon;
    drawingManager!.setDrawingMode(null);
    setDrawActive(false);

    const coordinates: PolygonCoord[] = overlay
      .getPath()
      .getArray()
      .map((ll: google.maps.LatLng): PolygonCoord => ({ lat: ll.lat(), lng: ll.lng() }));

    try {
      const created: SavedPolygon = await polygonService.create({ coordinates, city_id: activeCityId });
      polygonMap.set(created.id, created);
      overlayMap.set(created.id, overlay);
      attachHandlers(overlay, created);
    } catch {
      overlay.setMap(null);
    }
  });
});

onUnmounted((): void => {
  if (undoTimer) clearTimeout(undoTimer);
  cancelRubberband();
  drawingManager?.setMap(null);
  map = null;
  drawingManager = null;
  drawBtn = null; editBtn = null; deleteBtn = null; linkBtn = null; polygonZoneBtn = null;
  deleteMode = false; editMode = false; linkMode = false;
  editingOverlay = null; linkSource = null;
  polygonMap.clear();
  overlayMap.clear();
  neighborLines.clear();
  undoPending = null;
  polygonZoneMode.value = false;
  polygonZones.value = [];
  selectedPolygonZone.value = null;
});

// ─── Control group ───────────────────────────────────────────────────────────

// FontAwesome Pro is an optional dependency: it lives on a private registry, so a clone without a
// token installs everything else and skips it. The toolbar therefore resolves its glyphs at runtime
// and falls back to short text labels when the package isn't there — the buttons stay usable either
// way. Specifiers are variables + @vite-ignore so the bundler doesn't try to resolve them at build.
type IconDefinition = unknown;
interface FontAwesome {
  icon: (definition: IconDefinition) => { html: string[] } | undefined;
  glyphs: Record<string, IconDefinition>;
}
let fontAwesome: FontAwesome | null = null;

async function loadFontAwesome(): Promise<void> {
  const corePackage  = '@fortawesome/fontawesome-svg-core';
  const iconsPackage = '@fortawesome/pro-solid-svg-icons';
  try {
    const [core, glyphs] = await Promise.all([
      import(/* @vite-ignore */ corePackage),
      import(/* @vite-ignore */ iconsPackage),
    ]);
    fontAwesome = { icon: core.icon, glyphs: glyphs as Record<string, IconDefinition> };
  } catch {
    fontAwesome = null;
  }
}

// Returns the icon's SVG markup, or `fallback` text when FontAwesome isn't installed.
function glyph(name: string, fallback: string): string {
  const definition = fontAwesome?.glyphs[name];
  if (!fontAwesome || definition === undefined) return fallback;
  return fontAwesome.icon(definition)?.html[0] ?? fallback;
}

function addControlGroup(mapInstance: google.maps.Map, OverlayType: { POLYGON: google.maps.drawing.OverlayType }): void {
  const wrap = document.createElement('div');
  Object.assign(wrap.style, {
    margin: '10px 0 0 10px',
    display: 'flex',
    background: '#fff',
    borderRadius: '2px',
    boxShadow: '0 1px 4px rgba(0,0,0,.3)',
    overflow: 'hidden',
    fontFamily: 'Roboto, Arial, sans-serif',
    fontSize: '14px',
    fontWeight: '500',
    userSelect: 'none',
  });

  const base = { padding: '8px 10px', border: 'none', background: '#fff', color: '#444', cursor: 'pointer', lineHeight: '0', fontSize: '16px' };
  const sep  = { borderRight: '1px solid #e0e0e0' };

  drawBtn = makeBtn('Draw Polygon', glyph('faDrawPolygon', 'Draw'), { ...base, ...sep });
  drawBtn.addEventListener('click', async (): Promise<void> => {
    if (drawingManager?.getDrawingMode() !== null) {
      drawingManager?.setDrawingMode(null);
      setDrawActive(false);
    } else {
      await deactivateEdit();
      cancelLink();
      deactivatePolygonZoneMode();
      deleteMode = false; setDeleteActive(false);
      drawingManager?.setDrawingMode(OverlayType.POLYGON);
      setDrawActive(true);
    }
  });

  editBtn = makeBtn('Edit Polygon', glyph('faPenToSquare', 'Edit'), { ...base, ...sep });
  editBtn.addEventListener('click', async (): Promise<void> => {
    if (editMode) {
      await deactivateEdit();
    } else {
      drawingManager?.setDrawingMode(null); setDrawActive(false);
      cancelLink();
      deactivatePolygonZoneMode();
      deleteMode = false; setDeleteActive(false);
      editMode = true; setEditActive(true);
    }
  });

  linkBtn = makeBtn('Link Neighbors', glyph('faStreetView', 'Link'), { ...base, ...sep });
  linkBtn.addEventListener('click', async (): Promise<void> => {
    if (linkMode) {
      cancelLink();
    } else {
      drawingManager?.setDrawingMode(null); setDrawActive(false);
      await deactivateEdit();
      deactivatePolygonZoneMode();
      deleteMode = false; setDeleteActive(false);
      linkMode = true; setLinkActive(true);
      showAllNeighborLines();
    }
  });

  polygonZoneBtn = makeBtn('Polygon Zones', glyph('faGripHorizontal', 'Zones'), { ...base, ...sep });
  polygonZoneBtn.addEventListener('click', async (): Promise<void> => {
    if (polygonZoneMode.value) {
      deactivatePolygonZoneMode();
    } else {
      drawingManager?.setDrawingMode(null); setDrawActive(false);
      await deactivateEdit();
      cancelLink();
      deleteMode = false; setDeleteActive(false);
      polygonZoneMode.value = true; setPolygonZoneActive(true);
      // Zones are pre-loaded on mount; only re-fetch if somehow empty.
      if (polygonZones.value.length === 0) {
        polygonZones.value = await polygonZoneService.index().catch((): PolygonZone[] => []);
      }
      // Reveal existing polygon zone assignments now that mode is active.
      applyPolygonZoneColors();
    }
  });

  deleteBtn = makeBtn('Delete Polygon', glyph('faTrashCan', 'Delete'), { ...base });
  deleteBtn.addEventListener('click', async (): Promise<void> => {
    deleteMode = !deleteMode;
    if (deleteMode) {
      drawingManager?.setDrawingMode(null); setDrawActive(false);
      await deactivateEdit();
      cancelLink();
      deactivatePolygonZoneMode();
      showAllNeighborLines();
    } else {
      hideAllNeighborLines();
    }
    setDeleteActive(deleteMode);
  });

  wrap.appendChild(drawBtn);
  wrap.appendChild(editBtn);
  wrap.appendChild(linkBtn);
  wrap.appendChild(polygonZoneBtn);
  wrap.appendChild(deleteBtn);
  mapInstance.controls[google.maps.ControlPosition.LEFT_TOP].push(wrap);
}

function makeBtn(title: string, html: string, styles: Record<string, string>): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.title = title;
  btn.innerHTML = html;
  Object.assign(btn.style, styles);
  // The icon styles collapse line-height for a centred SVG; a text fallback needs it back.
  if (!html.startsWith('<')) {
    Object.assign(btn.style, { lineHeight: '1', fontSize: '13px' });
  }
  return btn;
}

// ─── Button state helpers ─────────────────────────────────────────────────────

function setDrawActive(on: boolean): void        { if (drawBtn)        { drawBtn.style.background        = on ? '#e8e8e8' : '#fff'; drawBtn.style.color        = '#444'; } }
function setEditActive(on: boolean): void        { if (editBtn)        { editBtn.style.background        = on ? '#e8e8e8' : '#fff'; editBtn.style.color        = '#444'; } }
function setLinkActive(on: boolean): void        { if (linkBtn)        { linkBtn.style.background        = on ? '#e8e8e8' : '#fff'; linkBtn.style.color        = '#444'; } }
function setPolygonZoneActive(on: boolean): void { if (polygonZoneBtn) { polygonZoneBtn.style.background = on ? '#e8e8e8' : '#fff'; polygonZoneBtn.style.color = '#444'; } }
function setDeleteActive(on: boolean): void      { if (deleteBtn)      { deleteBtn.style.background      = on ? '#B8422E' : '#fff'; deleteBtn.style.color      = on ? '#fff' : '#444'; } }

// ─── Edit mode ────────────────────────────────────────────────────────────────

async function deactivateEdit(): Promise<void> {
  if (!editingOverlay) { editMode = false; setEditActive(false); return; }

  const { overlay, id } = editingOverlay;
  editingOverlay = null; editMode = false; setEditActive(false);
  overlay.setOptions({ editable: false, draggable: false });

  const coordinates: PolygonCoord[] = overlay.getPath().getArray()
    .map((ll: google.maps.LatLng): PolygonCoord => ({ lat: ll.lat(), lng: ll.lng() }));
  try { await polygonService.update(id, { coordinates }); } catch { /* stay as-drawn */ }
}

// ─── Link mode ────────────────────────────────────────────────────────────────

function showAllNeighborLines(): void {
  for (const line of neighborLines.values()) line.setMap(map);
}

function hideAllNeighborLines(): void {
  for (const line of neighborLines.values()) line.setMap(null);
}

// Show only lines that touch polygonId; hide the rest.
function focusNeighborLines(polygonId: string): void {
  for (const [key, line] of neighborLines) {
    line.setMap(key.includes(polygonId) ? map : null);
  }
}

function cancelLink(): void {
  cancelRubberband();
  if (linkSource) {
    linkSource.overlay.setOptions({ fillOpacity: 0.15 });
    linkSource = null;
  }
  hideAllNeighborLines();
  linkMode = false;
  setLinkActive(false);
}

function cancelRubberband(): void {
  rubberband?.setMap(null);
  rubberband = null;
  if (mousemoveListener) { google.maps.event.removeListener(mousemoveListener); mousemoveListener = null; }
  if (mapClickListener)  { google.maps.event.removeListener(mapClickListener);  mapClickListener  = null; }
  for (const l of overlayMousemoveListeners) google.maps.event.removeListener(l);
  overlayMousemoveListeners = [];
}

function startRubberband(from: SavedPolygon): void {
  if (!map) return;
  const origin = centroid(from.coordinates);

  rubberband = new google.maps.Polyline({
    path: [origin, origin],
    strokeColor: '#555',
    strokeOpacity: 0,
    strokeWeight: 0,
    icons: [{
      icon: { path: 'M 0,-1 0,1', strokeOpacity: 0.7, strokeWeight: 2, scale: 3 },
      offset: '0',
      repeat: '12px',
    }],
    map,
  });

  const updatePath = (e: google.maps.MapMouseEvent): void => {
    if (e.latLng) rubberband?.setPath([origin, { lat: e.latLng.lat(), lng: e.latLng.lng() }]);
  };

  mousemoveListener = map.addListener('mousemove', updatePath);

  // Polygons absorb map-level mousemove events, so mirror the same handler onto
  // every overlay so the rubberband tracks the cursor even when crossing polygons.
  for (const overlay of overlayMap.values()) {
    overlayMousemoveListeners.push(overlay.addListener('mousemove', updatePath));
  }

  // Left-click on empty map cancels the pending link source but stays in link mode.
  mapClickListener = map.addListener('click', (): void => {
    cancelRubberband();
    if (linkSource) { linkSource.overlay.setOptions({ fillOpacity: 0.15 }); linkSource = null; }
    showAllNeighborLines();
  });

  // Right-click anywhere breaks the rubberband and exits link mode entirely.
  map.addListener('rightclick', (): void => {
    cancelRubberband();
    if (linkSource) { linkSource.overlay.setOptions({ fillOpacity: 0.15 }); linkSource = null; }
    showAllNeighborLines();
    linkMode = false;
    setLinkActive(false);
  });
}

// ─── Polygon zone mode ────────────────────────────────────────────────────────

function deactivatePolygonZoneMode(): void {
  if (!polygonZoneMode.value) return;
  polygonZoneMode.value = false;
  setPolygonZoneActive(false);
  selectedPolygonZone.value = null;
  showCreatePolygonZone.value = false;
  editingZoneId.value = null;
  resetPolygonColors();
}

function selectPolygonZone(zone: PolygonZone): void {
  selectedPolygonZone.value = zone;
  showCreatePolygonZone.value = false;
}

// Apply each polygon's zone color; unassigned polygons keep the default red at low opacity.
function applyPolygonZoneColors(): void {
  for (const [polygonId, polygon] of polygonMap) {
    const overlay = overlayMap.get(polygonId);
    if (!overlay) continue;
    if (polygon.polygon_zone_id) {
      const color = polygonZones.value.find((z: PolygonZone) => z.id === polygon.polygon_zone_id)?.color ?? '#B8422E';
      overlay.setOptions({ fillColor: color, strokeColor: color, fillOpacity: FILL_OPACITY_ZONED });
    } else {
      overlay.setOptions({ fillColor: '#B8422E', strokeColor: '#B8422E', fillOpacity: FILL_OPACITY_DEFAULT });
    }
    refreshNeighborLinesFor(polygonId);
  }
}

function resetPolygonColors(): void {
  for (const [, overlay] of overlayMap) {
    overlay.setOptions({ fillColor: '#B8422E', strokeColor: '#B8422E', fillOpacity: FILL_OPACITY_DEFAULT });
  }
}

function startEditZone(zone: PolygonZone): void {
  editingZoneId.value   = zone.id;
  editingZoneName.value = zone.name;
  editingZoneColor.value = zone.color;
  showCreatePolygonZone.value = false;
}

function cancelEditZone(): void {
  editingZoneId.value = null;
}

function cancelCreateZone(): void {
  showCreatePolygonZone.value = false;
  newPolygonZoneColor.value = ZONE_COLORS[0];
}

function openCreateZone(): void {
  editingZoneId.value = null;
  showCreatePolygonZone.value = true;
}

async function submitEditZone(): Promise<void> {
  if (!editingZoneId.value || !editingZoneName.value.trim()) return;
  savingZone.value = true;
  try {
    const updated: PolygonZone = await polygonZoneService.update(editingZoneId.value, {
      name: editingZoneName.value.trim(),
      color: editingZoneColor.value,
    });
    polygonZones.value = polygonZones.value.map((z: PolygonZone) => z.id === updated.id ? updated : z);
    // If this zone is currently selected, keep its color in sync.
    if (selectedPolygonZone.value?.id === updated.id) {
      selectedPolygonZone.value = updated;
    }
    // Repaint polygons that belong to this zone.
    for (const [polygonId, polygon] of polygonMap) {
      if (polygon.polygon_zone_id === updated.id) {
        overlayMap.get(polygonId)?.setOptions({ fillColor: updated.color, strokeColor: updated.color, fillOpacity: FILL_OPACITY_ZONED });
      }
    }
    editingZoneId.value = null;
  } catch { /* leave form open */ } finally {
    savingZone.value = false;
  }
}

async function submitCreatePolygonZone(): Promise<void> {
  if (!newPolygonZoneName.value.trim()) return;
  creatingPolygonZone.value = true;
  try {
    const created: PolygonZone = await polygonZoneService.create({
      name: newPolygonZoneName.value.trim(),
      color: newPolygonZoneColor.value,
    });
    polygonZones.value = [...polygonZones.value, created];
    selectPolygonZone(created);
    newPolygonZoneName.value = '';
    newPolygonZoneColor.value = ZONE_COLORS[0];
    showCreatePolygonZone.value = false;
  } catch { /* creation failed */ } finally {
    creatingPolygonZone.value = false;
  }
}

// ─── Neighbor lines ───────────────────────────────────────────────────────────

const LINE_PALETTE: string[] = [
  '#2196F3', // blue
  '#4CAF50', // green
  '#FF9800', // orange
  '#9C27B0', // purple
  '#00BCD4', // cyan
  '#E91E63', // pink
  '#8BC34A', // light green
  '#FF5722', // deep orange
  '#607D8B', // blue-grey
  '#795548', // brown
];

// Derives a stable color from a polygon's ID so every line it owns shares the same color.
function polygonColor(polygonId: string): string {
  let hash = 0;
  for (let i = 0; i < polygonId.length; i++) hash = (hash * 31 + polygonId.charCodeAt(i)) & 0x7fffffff;
  return LINE_PALETTE[hash % LINE_PALETTE.length];
}

// Returns the polygon's zone color if one is assigned, otherwise falls back to the hash color.
function resolvePolygonColor(polygon: SavedPolygon): string {
  if (polygon.polygon_zone_id) {
    const zone = polygonZones.value.find((z: PolygonZone) => z.id === polygon.polygon_zone_id);
    if (zone) return zone.color;
  }
  return polygonColor(polygon.id);
}

// Re-colors every neighbor line that touches polygonId using that polygon's own zone color.
// This ensures the source polygon's color always wins regardless of what the target has.
function refreshNeighborLinesFor(polygonId: string): void {
  const polygon = polygonMap.get(polygonId);
  if (!polygon) return;
  const color = resolvePolygonColor(polygon);
  for (const [key, line] of neighborLines) {
    if (key.includes(polygonId)) line.setOptions({ strokeColor: color });
  }
}

function pairKey(a: string, b: string): string {
  return a < b ? `${a}:${b}` : `${b}:${a}`;
}

function centroid(coords: PolygonCoord[]): { lat: number; lng: number } {
  return {
    lat: coords.reduce((s, c) => s + c.lat, 0) / coords.length,
    lng: coords.reduce((s, c) => s + c.lng, 0) / coords.length,
  };
}

function renderNeighborLine(a: SavedPolygon, b: SavedPolygon): void {
  const key = pairKey(a.id, b.id);
  if (neighborLines.has(key)) return;
  const line = new google.maps.Polyline({
    path: [centroid(a.coordinates), centroid(b.coordinates)],
    strokeColor: resolvePolygonColor(a),
    strokeOpacity: 0.7,
    strokeWeight: 1.5,
    clickable: true,
    // Lines are hidden by default; showAllNeighborLines/focusNeighborLines control visibility.
    map: null,
  });

  line.addListener('mouseover', (): void => {
    if (!deleteMode) return;
    line.setOptions({ strokeWeight: 6, strokeOpacity: 1 });
  });

  line.addListener('mouseout', (): void => {
    line.setOptions({ strokeWeight: 1.5, strokeOpacity: 0.7 });
  });

  line.addListener('click', async (): Promise<void> => {
    if (!deleteMode) return;
    const [polygonId, neighborId] = key.split(':');
    try {
      await neighborService.delete(polygonId, neighborId);
      line.setMap(null);
      neighborLines.delete(key);
    } catch { /* leave the line if the API call failed */ }
  });

  neighborLines.set(key, line);
}

async function removeNeighborLinesFor(polygonId: string): Promise<void> {
  const keys = [...neighborLines.keys()].filter(k => k.includes(polygonId));
  await Promise.all(keys.map(async (key): Promise<void> => {
    const [a, b] = key.split(':');
    try { await neighborService.delete(a, b); } catch { /* already gone via cascade */ }
    neighborLines.get(key)?.setMap(null);
    neighborLines.delete(key);
  }));
}

// ─── Per-polygon click handler ────────────────────────────────────────────────

function attachHandlers(overlay: google.maps.Polygon, polygon: SavedPolygon): void {
  overlay.addListener('click', async (): Promise<void> => {

    if (deleteMode) {
      try {
        await removeNeighborLinesFor(polygon.id);
        await polygonService.delete(polygon.id);
        overlay.setMap(null);
        polygonMap.delete(polygon.id);
        overlayMap.delete(polygon.id);
        showUndoToast(polygon.coordinates, polygon.city_id);
      } catch { /* polygon stays visible */ }
      return;
    }

    if (editMode) {
      if (editingOverlay && editingOverlay.overlay !== overlay) {
        await deactivateEdit();
        editMode = true; setEditActive(true);
      }
      editingOverlay = { overlay, id: polygon.id };
      overlay.setOptions({ editable: true, draggable: true });
      return;
    }

    if (linkMode) {
      if (!linkSource) {
        // First click — highlight this polygon and show only its existing connections.
        linkSource = { overlay, polygon };
        overlay.setOptions({ fillOpacity: 0.4 });
        startRubberband(polygon);
        focusNeighborLines(polygon.id);
      } else if (linkSource.polygon.id === polygon.id) {
        // Clicking source again cancels the pending link — restore all lines.
        cancelRubberband();
        overlay.setOptions({ fillOpacity: 0.15 });
        linkSource = null;
        showAllNeighborLines();
      } else {
        // Subsequent click — create the link but keep linkSource on polygon (1) so the
        // rubberband stays anchored there; the user can keep clicking targets without
        // having to re-select the source each time.
        const src = linkSource.polygon;

        const key = pairKey(src.id, polygon.id);
        if (!neighborLines.has(key)) {
          renderNeighborLine(src, polygon);
        }
        // Refresh focus so the newly drawn line becomes visible immediately.
        focusNeighborLines(src.id);

        // Fire-and-forget — backend is idempotent (ON CONFLICT DO NOTHING).
        // Never roll back: the line stays whether the API succeeds or the pair already exists.
        neighborService.create({ polygon_id: src.id, neighbor_polygon_id: polygon.id }).catch(() => { /* already exists */ });
      }
      return;
    }

    // Polygon zone mode: paint the selected zone onto clicked polygons.
    // Capture zone before the await so it cannot change mid-flight.
    // selectedPolygonZone is intentionally NOT cleared after each click — the user
    // can keep clicking polygons to paint them all with the same zone.
    if (polygonZoneMode.value) {
      const zone = selectedPolygonZone.value;
      if (!zone) return;
      try {
        const updated: SavedPolygon = await polygonService.assignPolygonZone(polygon.id, zone.id);
        polygonMap.set(updated.id, updated);
        overlay.setOptions({ fillColor: zone.color, strokeColor: zone.color, fillOpacity: FILL_OPACITY_ZONED });
        refreshNeighborLinesFor(updated.id);
      } catch { /* assignment failed silently */ }
    }
  });
}

// ─── Undo toast ───────────────────────────────────────────────────────────────

function showUndoToast(coords: PolygonCoord[], cityId: string | null): void {
  if (undoTimer) clearTimeout(undoTimer);
  undoPending = { coords, cityId };
  undoVisible.value = true;
  undoTimer = setTimeout(dismissUndo, 6000);
}

function dismissUndo(): void {
  if (undoTimer) clearTimeout(undoTimer);
  undoVisible.value = false; undoPending = null; undoTimer = null;
}

async function handleUndo(): Promise<void> {
  if (!undoPending || !map) return;
  const { coords, cityId } = undoPending;
  dismissUndo();
  try {
    const created: SavedPolygon = await polygonService.create({ coordinates: coords, city_id: cityId ?? undefined });
    polygonMap.set(created.id, created);
    const overlay = new google.maps.Polygon({
      paths: coords, fillColor: '#B8422E', fillOpacity: 0.15,
      strokeColor: '#B8422E', strokeWeight: 2, map,
    });
    overlayMap.set(created.id, overlay);
    attachHandlers(overlay, created);
  } catch { /* undo failed */ }
}

// ─── Location picker ──────────────────────────────────────────────────────────

function onLocationConfirmed(lat: number, lng: number, cityId: string): void {
  showModal.value = false;
  activeCityId = cityId;
  map?.setCenter({ lat, lng });
  map?.setZoom(12);
}
</script>

<template>
  <div class="map-shell">
    <div ref="mapEl" class="map" />
    <LocationPickerModal v-if="showModal" @confirm="onLocationConfirmed" />

    <!-- Polygon zone panel — visible only when polygon zone mode is active -->
    <div v-if="polygonZoneMode" class="polygon-zone-panel">
      <div class="polygon-zone-panel-header">
        <span class="polygon-zone-panel-title">Polygon Zones</span>
        <button class="polygon-zone-panel-close" @click="deactivatePolygonZoneMode">✕</button>
      </div>

      <div class="polygon-zone-list">
        <div
          v-for="zone in polygonZones"
          :key="zone.id"
          class="polygon-zone-item"
          :class="{ 'polygon-zone-item--active': selectedPolygonZone?.id === zone.id }"
        >
          <button class="polygon-zone-item-select" @click="selectPolygonZone(zone)">
            <span class="polygon-zone-swatch" :style="{ background: zone.color }" />
            <span class="polygon-zone-item-name">{{ zone.name }}</span>
          </button>
          <button
            class="polygon-zone-edit-btn"
            :class="{ 'polygon-zone-edit-btn--active': editingZoneId === zone.id }"
            title="Edit zone"
            @click.stop="editingZoneId === zone.id ? cancelEditZone() : startEditZone(zone)"
          >✎</button>
        </div>
        <p v-if="polygonZones.length === 0" class="polygon-zone-empty">No polygon zones yet</p>
      </div>

      <div class="polygon-zone-footer">
        <button
          class="polygon-zone-new-btn"
          :class="{ 'polygon-zone-new-btn--active': showCreatePolygonZone }"
          @click="showCreatePolygonZone ? cancelCreateZone() : openCreateZone()"
        >
          + New Polygon Zone
        </button>
      </div>

      <p v-if="selectedPolygonZone" class="polygon-zone-hint">
        Click a polygon to assign
        <span class="polygon-zone-hint-name" :style="{ color: selectedPolygonZone.color }">{{ selectedPolygonZone.name }}</span>
      </p>
      <p v-else class="polygon-zone-hint">Select a polygon zone above</p>
    </div>

    <!-- Floating form panel — appears to the right of the zone panel when creating or editing -->
    <Transition name="form-panel">
      <div v-if="polygonZoneMode && (showCreatePolygonZone || editingZoneId)" class="polygon-zone-form-panel">
        <div class="polygon-zone-panel-header">
          <span class="polygon-zone-panel-title">{{ editingZoneId ? 'Edit Zone' : 'New Zone' }}</span>
          <button class="polygon-zone-panel-close" @click="editingZoneId ? cancelEditZone() : cancelCreateZone()">✕</button>
        </div>
        <div class="polygon-zone-form">
          <input
            v-if="editingZoneId"
            v-model="editingZoneName"
            class="polygon-zone-input"
            placeholder="Zone name"
            maxlength="100"
            @keyup.enter="submitEditZone"
            @keyup.escape="cancelEditZone"
          />
          <input
            v-else
            v-model="newPolygonZoneName"
            class="polygon-zone-input"
            placeholder="Zone name"
            maxlength="100"
            @keyup.enter="submitCreatePolygonZone"
          />
          <div class="polygon-zone-color-grid">
            <button
              v-for="c in ZONE_COLORS"
              :key="c"
              class="polygon-zone-color-swatch"
              :class="{ 'polygon-zone-color-swatch--selected': (editingZoneId ? editingZoneColor : newPolygonZoneColor) === c }"
              :style="{ background: c }"
              :title="c"
              @click="editingZoneId ? (editingZoneColor = c) : (newPolygonZoneColor = c)"
            />
          </div>
          <div class="polygon-zone-form-actions">
            <button
              class="polygon-zone-cancel"
              @click="editingZoneId ? cancelEditZone() : cancelCreateZone()"
            >Cancel</button>
            <button
              v-if="editingZoneId"
              class="polygon-zone-save"
              :disabled="savingZone || !editingZoneName.trim()"
              @click="submitEditZone"
            >{{ savingZone ? '…' : 'Save' }}</button>
            <button
              v-else
              class="polygon-zone-save"
              :disabled="creatingPolygonZone || !newPolygonZoneName.trim()"
              @click="submitCreatePolygonZone"
            >{{ creatingPolygonZone ? '…' : 'Create' }}</button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="toast">
      <div v-if="undoVisible" class="undo-toast">
        <span>Polygon deleted</span>
        <button class="undo-btn" @click="handleUndo">Undo</button>
        <button class="dismiss-btn" @click="dismissUndo">✕</button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.map-shell {
  position: relative;
  width: 100%;
  height: calc(100vh - var(--topnav-height));
}
.map { width: 100%; height: 100%; }

/* ─── Polygon zone panel ──────────────────────────────────────── */
.polygon-zone-panel {
  position: absolute;
  top: 50px; /* sits below the toolbar strip */
  left: 10px;
  width: 220px;
  background: #fff;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  font-family: Roboto, Arial, sans-serif;
  font-size: 13px;
  z-index: 5;
  overflow: hidden;
}

.polygon-zone-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px 8px;
  border-bottom: 1px solid #eee;
}

.polygon-zone-panel-title {
  font-weight: 600;
  font-size: 13px;
  color: #222;
}

.polygon-zone-panel-close {
  background: none;
  border: none;
  color: #999;
  cursor: pointer;
  font-size: 12px;
  padding: 0;
  line-height: 1;
}
.polygon-zone-panel-close:hover { color: #444; }

.polygon-zone-list {
  max-height: 220px;
  overflow-y: auto;
  padding: 6px 0;
}

.polygon-zone-item {
  display: flex;
  align-items: center;
  width: 100%;
  background: none;
  transition: background 0.1s;
}
.polygon-zone-item:hover { background: #f5f5f5; }
.polygon-zone-item--active { background: #f0f0f0; font-weight: 600; }

.polygon-zone-item-select {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  padding: 7px 8px 7px 12px;
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  color: #333;
}

.polygon-zone-edit-btn {
  flex-shrink: 0;
  background: none;
  border: none;
  color: #bbb;
  cursor: pointer;
  font-size: 14px;
  padding: 7px 10px 7px 4px;
  line-height: 1;
  transition: color 0.1s;
}
.polygon-zone-edit-btn:hover { color: #555; }
.polygon-zone-edit-btn--active { color: #333; }

.polygon-zone-swatch {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 3px;
  flex-shrink: 0;
}

.polygon-zone-item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.polygon-zone-empty {
  padding: 10px 12px;
  color: #999;
  font-size: 12px;
  margin: 0;
}

.polygon-zone-footer {
  border-top: 1px solid #eee;
  padding: 8px 12px;
}

.polygon-zone-new-btn {
  background: none;
  border: none;
  color: #555;
  cursor: pointer;
  font-size: 13px;
  padding: 0;
}
.polygon-zone-new-btn:hover { color: #222; }
.polygon-zone-new-btn--active { color: #222; font-weight: 600; }

/* ─── Floating form panel ─────────────────────────────────────── */
.polygon-zone-form-panel {
  position: absolute;
  top: 50px;
  left: 238px; /* 10px + 220px panel + 8px gap */
  width: 210px;
  background: #fff;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  font-family: Roboto, Arial, sans-serif;
  font-size: 13px;
  z-index: 5;
  overflow: hidden;
}

.polygon-zone-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px 12px;
}

.form-panel-enter-active, .form-panel-leave-active { transition: opacity 0.15s, transform 0.15s; }
.form-panel-enter-from, .form-panel-leave-to { opacity: 0; transform: translateX(-6px); }

.polygon-zone-color-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 4px;
}

.polygon-zone-color-swatch {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 3px;
  border: 1.5px solid rgba(0, 0, 0, 0.12);
  cursor: pointer;
  padding: 0;
  transition: transform 0.1s, box-shadow 0.1s;
}
.polygon-zone-color-swatch:hover { transform: scale(1.15); }
.polygon-zone-color-swatch--selected {
  border-color: transparent;
  box-shadow: 0 0 0 2.5px #333;
  transform: scale(1.1);
}

.polygon-zone-input {
  width: 100%;
  border: 1px solid #ddd;
  border-radius: 3px;
  padding: 6px 8px;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}
.polygon-zone-input:focus { border-color: #aaa; }

.polygon-zone-form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.polygon-zone-cancel {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 12px;
  padding: 4px 0;
}
.polygon-zone-cancel:hover { color: #444; }

.polygon-zone-save {
  background: #222;
  color: #fff;
  border: none;
  border-radius: 3px;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
}
.polygon-zone-save:disabled { opacity: 0.4; cursor: default; }
.polygon-zone-save:not(:disabled):hover { background: #444; }

.polygon-zone-hint {
  padding: 6px 12px 10px;
  font-size: 11px;
  color: #888;
  margin: 0;
  border-top: 1px solid #eee;
}

.polygon-zone-hint-name {
  font-weight: 600;
}

/* ─── Undo toast ──────────────────────────────────────────────── */
.undo-toast {
  position: absolute;
  bottom: 32px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #222;
  color: #fff;
  border-radius: 4px;
  font-size: 14px;
  font-family: Roboto, Arial, sans-serif;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  white-space: nowrap;
  z-index: 10;
}
.undo-btn {
  background: none; border: none; color: #e8705e;
  font-weight: 700; font-size: 14px; cursor: pointer;
  padding: 0; text-transform: uppercase; letter-spacing: 0.5px;
}
.dismiss-btn {
  background: none; border: none; color: #aaa;
  font-size: 12px; cursor: pointer; padding: 0; line-height: 1;
}
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.2s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateX(-50%) translateY(8px); }
</style>
