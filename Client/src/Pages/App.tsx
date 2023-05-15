import { Link } from "react-router-dom";
import { useScore } from "../Hooks/CtrlHooks";
import { formatDate } from "../util";

const Scoreboard = () => {
  const { data, scoreLoading, scoreError, scoreUpdatedAt } = useScore();
  if (scoreLoading) return <div>Loading...</div>;
  if (scoreError) return <div>Error!</div>;
  return (
    <div>
      <table className="table-fixed w-full">
        <thead>
          <tr>
            <th>Teams</th>
            {data.services.map((service) => (
              <th className="text-center" key={service}>{service}</th>
            ))}
          </tr>
        </thead>
        <tbody className="bg-slate-50 dark:bg-zinc-800 shadow-md dark:border">
          {data.teams.map((team) => (
            <tr key={"ScoreRow" + team.name}>
              <td key={"Name" + team.name} className="p-2">
                <Link to={"/team/"+team.name}>{team.name}</Link>
              </td>
              {team.ups.map((up, i) =>
                up ? (
                  <td
                    key={team.name + i}
                    className="border dark:border-zinc-900 bg-green-500"
                  ></td>
                ) : (
                  <td
                    key={team.name + i}
                    className="border dark:border-zinc-900 bg-red-500"
                  ></td>
                )
              )}
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

const Leaderboard = () => {
  const { data, scoreLoading, scoreError } = useScore();
  if (scoreLoading || scoreError) return null;
  let teams = data.teams.concat();
  return (
    <div className="bg-slate-50 dark:bg-zinc-800 p-2 rounded-xl shadow-md dark:border">
      <h2 className="text-4xl">Leaderboard:</h2>
      <table className="table-auto">
        <tbody>
          {teams
            .sort((a, b) => {
              return b.score - a.score;
            })
            .map((team) => (
              <tr key={"Leader" + team.name}>
                <td className="font-medium"><Link to={"/team/"+team.name}>{team.name}:</Link></td>
                <td className="px-2">{team.score}</td>
              </tr>
            ))}
        </tbody>
      </table>
    </div>
  );
};
function App() {
  return (
    <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-gray-100">
      <div className="w-screen flex justify-center content-center">
        <div className="h-full">
          <h1 className="text-6xl font-extrabold my-3 text-center">
            Cyber Scoreboard {import.meta.env.DEV ? "(DEV)" : ""}
          </h1>
          <div className="px-10">
            <Scoreboard />
          </div>
          <div className="px-10 my-2">
            <Leaderboard />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
