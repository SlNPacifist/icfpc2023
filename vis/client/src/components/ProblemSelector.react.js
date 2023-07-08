'use strict';

export default function ProblemSelector({N, onChange}) {
    return (
        <select onChange={onChange}>
            {
                Array.from({length: N}, (_, i) => i + 1)
                    .map(i => (
                        <option key={i} value={i}>Problem {i}</option>
                    ))
            }
        </select>
    )
}
