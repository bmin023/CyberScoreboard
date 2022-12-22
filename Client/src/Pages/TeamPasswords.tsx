import { Link, useParams } from "react-router-dom";
import {
  useOverwritePasswords,
  useTeamPasswordGroups,
} from "../Hooks/CtrlHooks";

const TeamPasswords = () => {
  const { teamName } = useParams();
  const { groups, groupsLoading } = useTeamPasswordGroups(teamName);
  const { overwritePasswords } = useOverwritePasswords(teamName);
  if (groupsLoading) return <div>Loading...</div>;
  if (groups.length === 0)
    return (
      <div>
        <p>No Passwords For This Scenario!</p>
        <Link className="underline" to={`/team/${teamName}`}>
          Back to Team Page
        </Link>
      </div>
    );
  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const group = formData.get("group") as string;
    const passwords = formData.get("passwords") as string;
    if (group && passwords) {
      overwritePasswords(
        { group, passwords: { passwords: passwords } },
        {
          onError: (err) => {
            alert(err);
          },
          onSuccess: () => {
            alert("Passwords Updated!");
          },
        }
      );
    }
    // clear form
    e.currentTarget.reset();
  };
  return (
    <div className="dark:bg-zinc-900 dark:text-zinc-100 bg-slate-100 h-screen">
      <Link className="m-1 underline" to={`/team/${teamName}`}>
        Back to Team Page
      </Link>
      <form
        onSubmit={handleSubmit}
        className="bg-slate-300 dark:shadow-inner dark:bg-zinc-700 w-5/6 mx-auto mt-10 p-4 rounded shadow-lg"
      >
        <div className="flex mb-4 flex-row justify-center">
          <p className="text-xl font-medium my-auto mr-2">
            Set Passwords for Group:
          </p>
          <select
            name="group"
            className="text-center bg-slate-100 dark:bg-zinc-600 text-xl font-medium w-24 h-10 rounded shadow"
          >
            {groups.map((group) => (
              <option className="appearance-none" key={group} value={group}>
                {group}
              </option>
            ))}
          </select>
          <input
            type="submit"
            value="Submit"
            className="bg-slate-100 dark:bg-zinc-600 px-4 mx-2 rounded shadow text-xl font-medium"
          />
        </div>
        <textarea
          name="passwords"
          className="w-full h-96 rounded-lg shadow-inner font-mono dark:shadow-2xl bg-slate-50 dark:bg-zinc-800 p-2 outline-none"
        />
        <p className="text-sm font-light">
          Input passwords in the format 'username:password'. Usernames and
          passwords must contain only alphanumeric characters along with these
          special characters: !@#?%&. This is to protect the scoreboard. One
          username password pair per line.
        </p>
      </form>
    </div>
  );
};

export default TeamPasswords;
