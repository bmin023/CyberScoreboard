import { useEffect } from "react";
import { Link, useParams } from "react-router-dom";
import { useTeamInjects, useTeamScore, useTime } from "../Hooks/CtrlHooks";
import { InjectDesc } from "../types";
import { formatDate } from "../util";

const Scoreboard = () => {
  const { teamName } = useParams();
  const { data, scoreLoading, scoreError, scoreUpdatedAt } =
    useTeamScore(teamName);
  if (scoreLoading || !teamName) return <div>Loading...</div>;
  if (scoreError) return <div>Error!</div>;
  return (
    <div className="w-10/12 m-auto my-4">
      <table className="bg-slate-50 dark:bg-zinc-800 w-full shadow-md rounded-xl overflow-clip dark:border dark:border-zinc-700">
        <tbody>
          {data.scores.map((score, index) => (
            <tr key={data.services[index]}>
              <td
                className={
                  "p-2 w-0 border-b font-medium text-xl dark:border-zinc-700 dark:text-slate-50 " +
                  (score.up ? "bg-green-500" : "bg-red-500")
                }
              >
                {data.services[index]}
              </td>
              {score.history.map((up, i) =>
                up ? (
                  <td
                    key={data.services[index] + i}
                    className="bg-green-500 border-b dark:border-zinc-700"
                  ></td>
                ) : (
                  <td
                    key={data.services[index] + i}
                    className="bg-red-500 border-b dark:border-zinc-700"
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

const InjectHolder = ({ title, children }: { title: string, children: JSX.Element }) => {
  return (
    <div className="my-5">
      <h2 className="text-2xl text-center font-bold mb-1">{title}</h2>
      <div className="bg-slate-50 dark:bg-zinc-800 w-full shadow-md rounded-xl overflow-clip p-1 border border-zinc-700">
        {children}
      </div>
    </div>
  );
}

const Injects = () => {
  const { teamName } = useParams();
  const { injects, injectsLoading } = useTeamInjects(teamName);
  const { time } = useTime();

  const timeRemaining = (inject: InjectDesc) => {
    const endTime = inject.start + inject.duration;
    const timeLeft = endTime - time.minutes;
    const plural = Math.abs(timeLeft) === 1 ? "" : "s";
    if (timeLeft === 0) return "a few seconds ago";
    if (timeLeft === 1) return "in a few seconds"
    if (timeLeft < 0) return `${-timeLeft} minute${plural} ago`;
    return `in ${timeLeft} minute${plural}`;
  }

  if (injectsLoading) return <div>Loading...</div>;
  return (
    <div className="w-10/12 m-auto my-4">
      {injects.active_injects.length > 0 &&
        <InjectHolder title="Active Injects">
          <ul>
            {injects.active_injects.map((inject) => (
              <li key={inject.uuid}>
                <Link className="flex text-lg font-semibold hover:underline underline-offset-2 px-5" to={`/team/${teamName}/inject/${inject.uuid}`}>
                  <p className="flex-grow">
                    {inject.name}
                    {!inject.sticky &&
                      <span className="ml-1 text-sm font-light">
                        {!inject.file_type || inject.file_type.length > 0 ? "Due" : "Ending"} {timeRemaining(inject)}
                      </span>}
                  </p>
                  {injects.completed_injects.find((response) => response.inject_uuid === inject.uuid) &&
                    <p className="text-right text-sm font-mono py-1">
                      Submitted
                    </p>
                  }
                </Link>
              </li>
            ))}
          </ul>
        </InjectHolder>
      }
      {injects.completed_injects.length > 0 &&
        <InjectHolder title="Submission History">
          <ul>
            {injects.completed_injects.sort((a, b) => (b.upload_time - a.upload_time)).map((response) => (
              <li className="underline-offset-2 px-5" key={response.uuid}>
                <p>
                  Submitted {response.filename} to <Link to={`/team/${teamName}/inject/${response.inject_uuid}`} className="font-semibold underline">{response.name}</Link> at {formatDate(new Date(response.upload_time))} <span className="text-red-500">{response.late ? "(Late)" : ""}</span>
                </p>
              </li>
            ))}
          </ul>
        </InjectHolder>
      }
    </div>
  );
}

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
      <Injects />
    </div>
  );
};

export default TeamScore;
