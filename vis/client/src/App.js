import './App.css';
import {useState} from 'react';
import { XYFrame } from "semiotic";

import defaultProblem from './data/problem-58.json';
import defaultSolution from './solutions/problem-58.json';
import ProblemSelector from './components/ProblemSelector.react';

const N = 91;

const COLOR_CODING_BAD = ['#af1f28', '#cc2127', '#e43025', '#ef6023', '#f99e25', '#fece22'];
const COLOR_CODING_GOOD = ['#dbe237', '#99ca3b', '#43b749', '#2db672'];

function distSqr(x1, y1, x2, y2) {
  const dx = x1 - x2;
  const dy = y1 - y2;
  return dx * dx + dy * dy;
}

function dot(x1, y1, x2, y2) {
  return x1 * x2 + y1 * y2;
}

function cross(x1, y1, x2, y2) {
  return x1 * y2 - x2 * y1;
}

function distPointToSegmentSqr(px, py, x1, y1, x2, y2) {
  if (dot(px - x1, py - y1, x2 - x1, y2 - y1) < 0) {
    return distSqr(px, py, x1, y1);
  }
  if (dot(px - x2, py - y2, x1 - x2, y1 - y2) < 0) {
    return distSqr(px, py, x2, y2);
  }
  const t = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
  return t * t / ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1));
}

function score(attendee, musician, placement) {
  return Math.ceil(1000000.0 * attendee.tastes[musician] / distSqr(attendee.x, attendee.y, placement.x, placement.y));
}

function getAngleComparator(p0) {
  return (p1, p2) => {
    const a1 = Math.atan2(p1.y - p0.y, p1.x - p0.x);
    const a2 = Math.atan2(p2.y - p0.y, p2.x - p0.x);
    return a1 - a2;
  }
}

const getFrameProps = ({problem = defaultProblem, solution = defaultSolution}) => {
  const attendees = problem.attendees.map((at, index) => ({
    ...at,
    type: 'attendee',
    index,
    color: "#003f5c",
    radius: 2.0,
  }));

  const placements = solution.placements.map((p, index) => ({
    ...p,
    type: 'placement',
    index,
    color: "#d45087",
    radius: 5.0,
    instrument: problem.musicians[index],
  }));

  const pillars = problem.pillars.map((pillar, index) => ({
    ...pillar,
    type: 'pillar',
    x: pillar.center[0],
    y: pillar.center[1],
    index,
  }));


  const scores = Array.from(Array(attendees.length), () => new Array(placements.length));
  const isVisible = Array.from(Array(attendees.length), () => new Array(placements.length));
  for (let i = 0; i < placements.length; i++) {
    const msc = [...placements.slice(0, i), ...placements.slice(i + 1)];
    const att = [...attendees];
    const compare = getAngleComparator(placements[i]);
    msc.sort(compare);
    att.sort(compare);

    // keep that previous angle
    let m = -1;
    for (let j = 0; j < att.length; j++) {
      let aa = Math.atan2(att[j].y - placements[i].y, att[j].x - placements[i].x);
      let mn = m;
      while (true) {
        mn = mn + 1;
        let ma = Math.atan2(msc[mn % msc.length].y - placements[i].y, msc[mn % msc.length].x - placements[i].x);
        if (mn === msc.length || ma > aa) {
          break;
        }
      }
      m = mn - 1;
      isVisible[att[j].index][i] = distPointToSegmentSqr(
        msc[mn % msc.length].x, msc[mn % msc.length].y,
        placements[i].x, placements[i].y,
        att[j].x, att[j].y,
      ) >= 25 && distPointToSegmentSqr(
        msc[(m + msc.length) % msc.length].x, msc[(m + msc.length) % msc.length].y,
        placements[i].x, placements[i].y,
        att[j].x, att[j].y,
      ) >= 25;
    }
  }

  for (let i = 0; i < attendees.length; i++) {
    // placements.sort((a, b) => {
    //   return distSqr(attendees[i].x, attendees[i].y, a.x, a.y) - distSqr(attendees[i].x, attendees[i].y, b.x, b.y);
    // })
    for (let j = 0; j < placements.length; j++) {
      // let isVisible = true;
      // for (let k = 0; k < placements.length; k++) {
      //   if (k !== j && distPointToSegmentSqr(placements[k].x, placements[k].y, attendees[i].x, attendees[i].y, placements[j].x, placements[j].y) <= 25) {
      //     isVisible = false;
      //   }
      // }
      scores[i][j] = isVisible[i][j] ? score(attendees[i], problem.musicians[j], placements[j]) : 0;
      if (i === 5 && j === 1) {
        console.log(attendees[i]);
        console.log(problem.musicians[j]);
        console.log(placements[j]);
        const x = 1000000.0 * attendees[i].tastes[problem.musicians[j]];
        const y = distSqr(attendees[i].x, attendees[i].y, placements[j].x, placements[j].y);
        console.log(x, y);
      }
    }
  }

  const maxD = Math.max(...attendees.flatMap(({x, y}) => [x, y]));

  const attendeeScores = scores.map((s, idx) => {
    return s.reduce((p, c) => p + c, 0.0);
  });
  const placementScores = Array(attendees.length).fill(0);
  scores.forEach(s => {
    s.forEach((score, index) => placementScores[index] += score);
  })

  const getScore = ({data}) => {
    if (data.type === 'attendee') {
      return attendeeScores[data.index];
    }
    if (data.type === 'placement') {
      return placementScores[data.index];
    }
  }

  function getColor({type, index, color}) {
    if (type === 'placement') {
      const score = placementScores[index];
      if (score < 0) {
        return '#af1f28';
      }
      if (score > 0) {
        return '#2db672';
      }
      return '#ccc';
    }
    return color;
  }

  return {
    scores,
    isVisible,
    frameProps: {
      xExtent: [0, maxD],
      yExtent: [0, maxD],
      lines: [{
        coordinates: [
          {x: problem.stage_bottom_left[0], y: problem.stage_bottom_left[1]},
          {x: problem.stage_bottom_left[0] + problem.stage_width, y: problem.stage_bottom_left[1]},
          {x: problem.stage_bottom_left[0] + problem.stage_width, y: problem.stage_bottom_left[1] + problem.stage_height},
          {x: problem.stage_bottom_left[0], y: problem.stage_bottom_left[1] + problem.stage_height},
          {x: problem.stage_bottom_left[0], y: problem.stage_bottom_left[1]},
        ],
        color: "#ff0000",
        strokeDasharray: '',
      }, {
        coordinates: [
          {x: problem.stage_bottom_left[0] + 5, y: problem.stage_bottom_left[1] + 5},
          {x: problem.stage_bottom_left[0] + problem.stage_width - 5, y: problem.stage_bottom_left[1] + 5},
          {x: problem.stage_bottom_left[0] + problem.stage_width - 5, y: problem.stage_bottom_left[1] + problem.stage_height - 5},
          {x: problem.stage_bottom_left[0] + 5, y: problem.stage_bottom_left[1] + problem.stage_height - 5},
          {x: problem.stage_bottom_left[0] + 5, y: problem.stage_bottom_left[1] + 5},
        ],
        color: "#00ff00",
        strokeDasharray: '15',
      }],
      lineStyle: ({color, strokeDasharray}) => ({ stroke: color, strokeWidth: 2, strokeDasharray}),
      points: [...attendees, ...placements, ...pillars], //[{ y: 326, x: 0.23, size: 55, color: "#ac58e5", clarity: "SI2" }],
      size: [10000, 10000],
      xAccessor: "x",
      yAccessor: "y",
      pointStyle: function(e) { return { fill: e.color, fillOpacity: .9 } },
      customPointMark: function(e) {
        const color = getColor(e.d);
        // does not work with canvas
        return ( <g><circle r={e.d.radius ? e.xScale(e.d.radius) : 1} fill={color} /></g> );
      },
      title: "Diamonds: Carat vs Price",
      axes: [{ orient: "bottom", label: "X" }, { label: "Y", orient: "left" }],
      canvasPoints: false,
      hoverAnnotation: true,
      tooltipContent: d => {
        return (
          <div className="App-tooltip-content">
            <p><b>Index: {d.index}</b></p>
            <p><b>Score: {getScore(d)}</b></p>
            <p>X: {d.y}</p>
            <p>Y: {d.x}</p>
            {d.data.type === 'placement' && (
              <p>
                Instrument: {d.instrument}
              </p>
            )}
          </div>
        );
      }
    }
  }
}

function App() {
  const [problem, setProblem] = useState();
  const [solution, setSolution] = useState();

  const onChange = (e) => {
    const value = e.target.value;
    setProblem(require(`./data/problem-${value}.json`))
    setSolution(require(`./solutions/problem-${value}.json`))
  }
  const {scores, isVisible, frameProps} = getFrameProps({problem, solution});

  return (
    <>
      <div className="App">
        <div className="App-selector">
          <ProblemSelector N={N} onChange={onChange} />
        </div>
        <XYFrame {...frameProps} className="App-xyframe"/>
        {/* {isVisible.map((s, idx) => {
          const score = s.reduce((p, c) => p + (c ? "1" : "0"), "");
          return (
            <div key={idx}>{score}</div>
          )
        })} */}
        {scores.map((s, idx) => {
          const score = s.reduce((p, c) => p + c, 0.0);
          return (
            <div key={idx}>Attendee {idx + 1}: {score}</div>
          )
        })}
      </div>
      <div>Total: {scores.reduce((p, c) => p + c.reduce((prev, cur) => prev + cur, 0), 0)}</div>
    </>
  );
}

export default App;
