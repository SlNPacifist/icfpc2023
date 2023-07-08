import './App.css';
import {useState} from 'react';
import { XYFrame } from "semiotic";

import defaultProblem from './data/problem-51.json';
import defaultSolution from './solutions/problem-51.json';
import ProblemSelector from './components/ProblemSelector.react';

const N = 55;

function distSqr(x1, y1, x2, y2) {
  const dx = x1 - x2;
  const dy = y1 - y2;
  return dx * dx + dy * dy;
}

function dot(x1, y1, x2, y2) {
  return x1 * x2 + y1 * y2;
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

const getFrameProps = ({problem = defaultProblem, solution = defaultSolution}) => {
const attendees = problem.attendees.map(at => ({
  ...at,
  color: "#003f5c",
}));

const placements = solution.placements.map(p => ({
  ...p,
  color: "#d45087",
}));

const scores = Array.from(Array(attendees.length), () => new Array(placements.length));
for (let i = 0; i < attendees.length; i++) {
  // placements.sort((a, b) => {
  //   return distSqr(attendees[i].x, attendees[i].y, a.x, a.y) - distSqr(attendees[i].x, attendees[i].y, b.x, b.y);
  // })
  for (let j = 0; j < placements.length; j++) {
    let isVisible = true;
    for (let k = 0; k < placements.length; k++) {
      if (k !== j && distPointToSegmentSqr(placements[k].x, placements[k].y, attendees[i].x, attendees[i].y, placements[j].x, placements[j].y) <= 25) {
        isVisible = false;
      }
    }
    scores[i][j] = isVisible ? score(attendees[i], problem.musicians[j], placements[j]) : 0;
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

return {
  scores,
  frameProps: {
    xExtent: [0, problem.room_width],
    yExtent: [0, problem.room_height],
    lines: [{
      coordinates: [
        {x: problem.stage_bottom_left[0], y: problem.stage_bottom_left[1]},
        {x: problem.stage_bottom_left[0] + problem.stage_width, y: problem.stage_bottom_left[1]},
        {x: problem.stage_bottom_left[0] + problem.stage_width, y: problem.stage_bottom_left[1] + problem.stage_height},
        {x: problem.stage_bottom_left[0], y: problem.stage_bottom_left[1] + problem.stage_height},
        {x: problem.stage_bottom_left[0], y: problem.stage_bottom_left[1]},
      ],
      color: "#ff0000"
    }],
    lineStyle: { stroke: "#ff0000", strokeWidth: 2 },
    points: [...attendees, ...placements], //[{ y: 326, x: 0.23, size: 55, color: "#ac58e5", clarity: "SI2" }],
    size: [1000, 800],
    xAccessor: "x",
    yAccessor: "y",
    pointStyle: function(e) { return { fill: e.color, fillOpacity: .9 } },
    title: "Diamonds: Carat vs Price",
    axes: [{ orient: "bottom", label: "X" }, { label: "Y", orient: "left" }],
    canvasPoints: true,
    hoverAnnotation: true,
    tooltipContent: d => {
      return (
        <div className="tooltip-content">
          <p>Price: ${d.y}</p>
          <p>Caret: {d.x}</p>
          <p>
            {d.coincidentPoints.length > 1 &&
              `+${d.coincidentPoints.length - 1} more diamond${(d.coincidentPoints
                .length > 2 &&
                "s") ||
                ""} here`}
          </p>
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
  const {scores, frameProps} = getFrameProps({problem, solution});

  return (
    <>
      <div className="App">
        <div className="App-selector">
          <ProblemSelector N={N} onChange={onChange} />
        </div>
        <XYFrame {...frameProps} />
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
