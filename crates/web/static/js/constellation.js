document.addEventListener('DOMContentLoaded', function() {

var W = window.innerWidth, H = window.innerHeight;
var CX = W / 2, CY = H / 2;
var MAX_D = Math.sqrt(CX * CX + CY * CY);
var MX = -1000, MY = -1000;
document.addEventListener('mousemove', function(e) { MX = e.clientX; MY = e.clientY; });

function dist(a, b) { return Math.sqrt((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)); }

function scatter(n, sparsity) {
  var out = [];
  for (var i = 0; i < n; i++) {
    var x, y, att = 0;
    do {
      x = Math.random() * W; y = Math.random() * H;
      var dc = Math.sqrt((x - CX) * (x - CX) + ((y - CY) * 1.2) * ((y - CY) * 1.2));
      if (Math.random() < Math.min(Math.pow(dc / (MAX_D * 0.45), sparsity), 1)) break;
      att++;
    } while (att < 30);
    out.push({ x: x, y: y });
  }
  return out;
}

// Single shared constellation
var svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
svg.setAttribute('width', W); svg.setAttribute('height', H);
document.getElementById('field').appendChild(svg);

var N = 100, CONNECT = 110, BREAK_D = 150, SPEED = 0.12, HOVER_R = 140;
var pts = scatter(N, 0.9);
var stars = [];
for (var i = 0; i < pts.length; i++) {
  var p = pts[i];
  var dc = dist(p, { x: CX, y: CY });
  var edge = Math.min(dc / (MAX_D * 0.4), 1);
  stars.push({
    x: p.x, y: p.y,
    r: 0.8 + Math.random() * 1.8,
    op: 0.025 + edge * 0.07,
    vx: (Math.random() - 0.5) * SPEED,
    vy: (Math.random() - 0.5) * SPEED,
    el: null
  });
}

for (var i = 0; i < stars.length; i++) {
  var s = stars[i];
  var c = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
  c.setAttribute('cx', s.x); c.setAttribute('cy', s.y); c.setAttribute('r', s.r);
  c.setAttribute('fill', 'rgba(190,155,80,' + s.op + ')');
  svg.appendChild(c); s.el = c;
}

var linePool = [];
var activeLines = {};

function lineKey(i, j) { return i < j ? i + '-' + j : j + '-' + i; }
function getLine() {
  if (linePool.length) return linePool.pop();
  var l = document.createElementNS('http://www.w3.org/2000/svg', 'line');
  l.setAttribute('stroke-width', '0.5');
  svg.insertBefore(l, svg.firstChild);
  return l;
}

for (var i = 0; i < N; i++) {
  for (var j = i + 1; j < N; j++) {
    if (dist(stars[i], stars[j]) < CONNECT) {
      var el = getLine();
      el.setAttribute('x1', stars[i].x); el.setAttribute('y1', stars[i].y);
      el.setAttribute('x2', stars[j].x); el.setAttribute('y2', stars[j].y);
      el.setAttribute('stroke', 'rgba(190,155,80,0.025)');
      activeLines[lineKey(i, j)] = { el: el };
    }
  }
}

function tick() {
  for (var i = 0; i < stars.length; i++) {
    var s = stars[i];
    var fx = s.vx, fy = s.vy;
    if (MX > 0) {
      var dx = MX - s.x, dy = MY - s.y;
      var dd = Math.sqrt(dx * dx + dy * dy);
      if (dd < 250 && dd > 5) {
        var pull = 0.4 / (dd * 0.3);
        fx += dx / dd * pull; fy += dy / dd * pull;
      }
    }
    s.x += fx; s.y += fy;
    if (s.x < -30 || s.x > W + 30) s.vx *= -1;
    if (s.y < -30 || s.y > H + 30) s.vy *= -1;
    s.vx *= 0.999; s.vy *= 0.999;
    if (Math.abs(s.vx) < 0.02) s.vx = (Math.random() - 0.5) * SPEED;
    if (Math.abs(s.vy) < 0.02) s.vy = (Math.random() - 0.5) * SPEED;
    s.el.setAttribute('cx', s.x); s.el.setAttribute('cy', s.y);
  }

  for (var key in activeLines) {
    var parts = key.split('-');
    var ii = parseInt(parts[0]), jj = parseInt(parts[1]);
    var dd = dist(stars[ii], stars[jj]);
    if (dd > BREAK_D) {
      activeLines[key].el.setAttribute('stroke', 'rgba(190,155,80,0)');
      linePool.push(activeLines[key].el);
      delete activeLines[key];
    } else {
      var stretch = (dd - CONNECT) / (BREAK_D - CONNECT);
      var fadeOp = stretch > 0 ? 0.025 * (1 - stretch * stretch) : 0.025;
      activeLines[key].el.setAttribute('x1', stars[ii].x); activeLines[key].el.setAttribute('y1', stars[ii].y);
      activeLines[key].el.setAttribute('x2', stars[jj].x); activeLines[key].el.setAttribute('y2', stars[jj].y);
      activeLines[key].el.setAttribute('stroke', 'rgba(190,155,80,' + Math.max(fadeOp, 0) + ')');
    }
  }

  for (var c = 0; c < 200; c++) {
    var ii = Math.floor(Math.random() * N), jj = Math.floor(Math.random() * N);
    if (ii === jj) continue;
    var key = lineKey(ii, jj);
    if (activeLines[key]) continue;
    if (dist(stars[ii], stars[jj]) < CONNECT) {
      var el = getLine();
      el.setAttribute('x1', stars[ii].x); el.setAttribute('y1', stars[ii].y);
      el.setAttribute('x2', stars[jj].x); el.setAttribute('y2', stars[jj].y);
      el.setAttribute('stroke', 'rgba(190,155,80,0.025)');
      activeLines[key] = { el: el };
    }
  }

  for (var i = 0; i < stars.length; i++) {
    var s = stars[i];
    var dd = dist(s, { x: MX, y: MY });
    if (dd < HOVER_R) {
      var t = 1 - dd / HOVER_R;
      var g = t * t * t;
      s.el.setAttribute('r', s.r + g * 3);
      s.el.setAttribute('fill', 'rgba(210,175,80,' + (s.op + g * 0.35) + ')');
      if (g > 0.2) s.el.style.filter = 'drop-shadow(0 0 ' + (g * 8) + 'px rgba(210,175,80,' + (g * 0.2) + '))';
      else s.el.style.filter = '';
    } else {
      s.el.setAttribute('r', s.r);
      s.el.setAttribute('fill', 'rgba(190,155,80,' + s.op + ')');
      s.el.style.filter = '';
    }
  }

  for (var key in activeLines) {
    var parts = key.split('-');
    var ii = parseInt(parts[0]), jj = parseInt(parts[1]);
    var mx = (stars[ii].x + stars[jj].x) / 2, my = (stars[ii].y + stars[jj].y) / 2;
    var dd = dist({ x: mx, y: my }, { x: MX, y: MY });
    if (dd < HOVER_R * 0.7) {
      var t = 1 - dd / (HOVER_R * 0.7);
      var g = t * t * t;
      activeLines[key].el.setAttribute('stroke', 'rgba(210,175,80,' + (0.025 + g * 0.1) + ')');
      activeLines[key].el.setAttribute('stroke-width', String(0.5 + g * 0.6));
    }
  }

  requestAnimationFrame(tick);
}
tick();

});
