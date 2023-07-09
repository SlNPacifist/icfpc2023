import './App.css';
import {useState, useEffect, useCallback, useRef, useLayoutEffect} from 'react';
import { XYFrame } from "semiotic";

import ProblemSelector from './components/ProblemSelector.react';
import COLORS from './colors.json';
import * as d3 from 'd3';

const N = 91;
let xScale;

function getInstrumentColor(instrument) {
  return COLORS[instrument % COLORS.length];
}

const defaultScore = {
  score: 0,
  attendee: [],
  musician: [],
};
const getFrameProps = ({problem, solution, score}) => {
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
    volume: solution.volumes[index],
    instrument: problem.musicians[index],
  }));

  const pillars = problem.pillars.map((pillar, index) => ({
    ...pillar,
    type: 'pillar',
    x: pillar.center[0],
    y: pillar.center[1],
    index,
  }));

  const maxD = Math.max(...attendees.flatMap(({x, y}) => [x, y]));

  const getScore = ({data}) => {
    if (data.type === 'attendee') {
      return score.attendee[data.index];
    }
    if (data.type === 'placement') {
      return score.musician[data.index];
    }
  }

  function getColor({type, index, color}) {
    if (type === 'placement') {
      const s = score.musician[index];
      if (s === undefined) {
        return '#dcdcdc';
      }
      if (s < 0) {
        return '#af1f28';
      }
      if (s > 0) {
        return '#2db672';
      }
      return '#ccc';
    }
    return color;
  }

  return {
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
      pointStyle: function(e) { return { fill: e.color, } },
      customPointMark: function(e) {
        const color = getColor(e.d);
        const instrumentColor = e.d.type === 'placement' ? getInstrumentColor(e.d.instrument) : '';
        xScale = e.xScale(1);
        return ( <g id={`${e.d.type}-${e.d.index}`} data-index={e.d.index} data-type={e.d.type}>
          <circle r={e.d.radius ? e.xScale(e.d.radius) : 1} fill={color} />
          {e.d.type=== 'placement' && e.d.radius > 3 && (
            <>
              <circle r={e.xScale(e.d.radius - 2)} fill={instrumentColor} stroke="#000000" strokeWidth={2} />
              <text alignmentBaseline="central" textAnchor="middle" style={{mixBlendMode: 'difference', filter: 'invert(1) grayscale(1) contrast(9)'}}>
                {e.d.instrument}
              </text>
            </>
          )}
        </g> );
      },
      title: "Diamonds: Carat vs Price",
      axes: [{ orient: "bottom", label: "X" }, { label: "Y", orient: "left" }],
      canvasPoints: false,
      hoverAnnotation: true,
      tooltipContent: d => {
        const score = getScore(d);
        return (
          <div className="App-tooltip-content">
            <p><b>Index: {d.index}</b></p>
            <p><b>Score: {Number.isFinite(score) && score.toLocaleString()}</b></p>
            <p>X: {d.y}</p>
            <p>Y: {d.x}</p>
            {d.data.type === 'placement' && (
              <>
                <p>
                  Instrument: {d.instrument}
                </p>
                <p>
                  Volume: {d.volume}
                </p>
              </>
            )}
          </div>
        );
      }
    }
  }
}

async function fetchApi(url) {
  const response = await fetch(url);
  if (!response.ok) {
    console.error(`Could not fetch ${url}`, response.status);
    return;
  }
  return response.json();
};

async function fetchScore(problemId, solution) {
  const response = await fetch(`/api/solution/${problemId}/score`, {
    method: 'POST',
    headers: {
      'Content-type': 'text/plain',
    },
    body: JSON.stringify(solution),
  });
  if (!response.ok) {
    console.error(response.status);
    return;
  }
  return response.json();
}

function App() {
  const [problem, setProblem] = useState();
  const [solution, setSolution] = useState();
  const [problemId, setProblemId] = useState(1);
  const [score, setScore] = useState(defaultScore);
  const dragStartCoords = useRef(null);
  const dragX = useRef(null);
  const dragY = useRef(null);

  const onChange = (e) => {
    const value = e.target.value;
    setProblemId(value);
  }

  useEffect(() => {
    (async () => {
      const [problem, solution] = await Promise.all([`/api/problem/${problemId}`, `/api/solution/${problemId}`].map(fetchApi));
      const score = await fetchScore(problemId, solution);
      setProblem(problem);
      setSolution(solution);
      setScore(score);
    })();
  }, [problemId]);
  const started = useCallback((element, event) => {
    const {type, index} = element.dataset;
    if (type !== 'placement') {
      return;
    }
    dragStartCoords.current = [event.x, event.y];
    dragX.current = 0;
    dragY.current = 0;
  }, []);
  const drag = useCallback((element, event) => {
    const [prevX, prevY] = dragStartCoords.current;
    const [translateX, translateY] = element.parentElement.attributes.transform.value.match(/([\d.]+),([\d.]+)/g)[0]
      .split(',')
      .map(Number);
    element.parentElement.attributes.transform.value = `translate(${translateX + event.x},${translateY + event.y})`;
    dragX.current += event.x;
    dragY.current += event.y;
  }, [solution]);
  const ended = useCallback((element, event) => {
    const {type, index} = element.dataset;
    if (type !== 'placement' || !dragStartCoords.current) {
      return;
    }

    const [prevX, prevY] = dragStartCoords.current;
    const shiftX = (dragX.current) / xScale;
    const shiftY = (dragY.current) / xScale;
    solution.placements[index].x += shiftX;
    solution.placements[index].y -= shiftY;
    const newSolution = {...solution};

    fetchScore(problemId, newSolution)
      .then((score) => {
        setSolution({...solution});
        setScore(score);
      });
    dragStartCoords.current = null;
  }, [solution]);

  useLayoutEffect(() => {
    if (!solution) return;
    solution.placements.forEach((placement, i) => {
    //   d3.drag
    //   .on("drag", function () {
    //     d3.select(this)
    //         .attr("x", d3.event.x)
    //         .attr("y", d3.event.y);
    // });
      d3.select(`#placement-${i}`).call(
        d3.drag()
          .on("start", function(e) {started(this, e)})
          .on('drag', function(e) {drag(this, e)})
          .on('end', function(e) {ended(this, e)})
      );
    });
  }, [solution]);

  const {frameProps} = (problem && solution && score && getFrameProps({problem, solution, score})) || {};
  const toggleHoverLayer = () => {
    document.body.classList.toggle('hiddenHoverLayer');
  }

  return (
    <>
      <div className="App">
        {frameProps && (<XYFrame {...frameProps} className="App-xyframe"/>)}
        {/* {isVisible.map((s, idx) => {
          const score = s.reduce((p, c) => p + (c ? "1" : "0"), "");
          return (
            <div key={idx}>{score}</div>
          )
        })} */}
        {score && score.attendee.map((s, idx) => {
          return (
            <div key={idx}>Attendee {idx + 1}: {s}</div>
          )
        })}
      </div>
      {score && <div>Total: {score.score}</div>}
      <div className="App-global-menu">
        <button onClick={toggleHoverLayer}>Toggle hover layer</button>
        <div className="App-selector">
          <ProblemSelector N={N} onChange={onChange} />
        </div>
        <p>CurrentScore: {score && score.score.toLocaleString()}</p>
      </div>
    </>
  );
}

export default App;
