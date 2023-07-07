import './App.css';
import { XYFrame } from "semiotic";

import problem from './data/problem-19.json';
import solution from './solutions/problem-19.json';

const attendees = problem.attendees.map(at => ({
  ...at,
  color: "#003f5c",
}));

const musicians = solution.placements.map(p => ({
  ...p,
  color: "#d45087",
}));

const frameProps = {
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
  points: [...attendees, ...musicians], //[{ y: 326, x: 0.23, size: 55, color: "#ac58e5", clarity: "SI2" }],
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
function App() {
  return (
    <div className="App">
      <XYFrame {...frameProps} />
    </div>
  );
}

export default App;
