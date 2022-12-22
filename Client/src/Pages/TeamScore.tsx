import { Link, useParams } from "react-router-dom";
import { useTeamScore } from "../Hooks/CtrlHooks";
import { formatDate } from "../util";

const Scoreboard = () => {
  const { teamName } = useParams();
  const { data, scoreLoading, scoreError, scoreUpdatedAt } =
    useTeamScore(teamName);
  if (scoreLoading) return <div>Loading...</div>;
  if (scoreError) return <div>Error!</div>;
  return (
    <div className="w-10/12 m-auto my-4">
      <table className="bg-slate-50 dark:bg-zinc-800 w-full shadow-md rounded-xl overflow-clip dark:border dark:rounded-none">
        <tbody>
          {data.scores.map((score, index) => (
            <tr key={data.services[index]}>
              <td
                className={
                  "p-2 w-0 border-b font-medium text-xl " +
                  (score.up ? "bg-green-500" : "bg-red-500")
                }
              >
                {data.services[index]}
              </td>
              {score.history.map((up, i) =>
                up ? (
                  <td
                    key={data.services[index] + i}
                    className="bg-green-500 border-b"
                  ></td>
                ) : (
                  <td
                    key={data.services[index] + i}
                    className="bg-red-500 border-b"
                  ></td>
                )
              )}
              <td className="p-2 w-0">{score.score}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <label className="text-sm">
        Last Updated: {formatDate(new Date(scoreUpdatedAt))}
      </label>
    </div>
  );
};

const TeamScore = () => {
  const { teamName } = useParams();
  const { data, scoreLoading, scoreError } = useTeamScore(teamName);
  if (scoreLoading)
    return (
      <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
        <div>Loading...</div>
      </div>
    );
  if (scoreError)
    return (
      <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
        <p>
          We encountered an error while retrieving scores for {teamName}. Is it
          spelled correctly? (Capitalization counts)
        </p>
      </div>
    );
  const getTotalScore = () => {
    let total = 0;
    data.scores.forEach((score) => {
      total += score.score;
    });
    return total;
  };
  return (
    <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
      <div className="flex">
        <Link className="underline m-1" to="/">Main Page</Link>
        <Link className="underline m-1" to={`/team/${teamName}/passwords`}>Passwords</Link>
      </div>
      <h1 className="text-4xl text-center font-bold">
        {teamName}'s Scoreboard
      </h1>
      <h2 className="text-2xl text-center font-medium">
        Total: {getTotalScore()}
      </h2>
      <Scoreboard />
    </div>
  );
};

export default TeamScore;
