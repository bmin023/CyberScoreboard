import React, { useEffect, useState } from "react";
import {
  useEditEnv,
  useDeleteEnv,
  useAddEnv,
  useDeleteTeam,
  useEditTeam,
  useAdminInfo,
  useAddTeam,
  useAdminPasswords,
  useWritePasswords,
  useDeletePasswords,
} from "../Hooks/CtrlHooks";
import NaiveShrinker from "./NaiveShrinker";
import { Shrinker } from "./Shrinker";

interface IEnvDisplayProps {
  team: string;
  env: [string, string];
}

const EnvDisplay: React.FC<IEnvDisplayProps> = ({ team, env }) => {
  const [name, setName] = useState(env[0]);
  const [value, setValue] = useState(env[1]);
  const { editEnv } = useEditEnv(team, env[0]);
  const { deleteEnv } = useDeleteEnv(team, env[0]);

  const hasChanged = () => {
    return name != env[0] || value != env[1];
  };

  return (
    <div className="flex ml-4 mt-0.5 text-sm font-mono">
      <Shrinker
        className="bg-slate-300 dark:bg-zinc-800 font-medium outline-none"
        value={name}
        onChange={(n) => setName(n)}
      />
      <span>:</span>
      <Shrinker
        className="bg-slate-300 ml-2 dark:bg-zinc-800 font-medium outline-none"
        value={value}
        onChange={(v) => setValue(v)}
      />
      <button
        className="ml-auto bg-slate-100 dark:bg-zinc-600 dark:disabled:bg-zinc-800 dark:disabled:bg-opacity-50 rounded-sm px-1 shadow-sm disabled:bg-slate-200 disabled:shadow-none disabled:text-gray-500 hover:shadow-lg active:shadow-none active:bg-slate-200"
        onClick={() => {
          editEnv({ name, value });
        }}
        disabled={!hasChanged()}
      >
        Save
      </button>
      <button
        className="outline outline-1 text-zinc-500 hover:text-red-500 ml-1 rounded-sm px-1"
        onClick={() => deleteEnv()}
      >
        Delete
      </button>
    </div>
  );
};

interface ITeamDisplayProps {
  team: {
    name: string;
    env: [string, string][];
  };
}

const PasswordDisplay: React.FC<{ team: string }> = ({ team }) => {
  const { passwords, passwordsError, passwordsLoading } =
    useAdminPasswords(team, (data) => {
      setSaved(true)
    });
  const { writePasswords } = useWritePasswords(team);
  const { deletePasswords } = useDeletePasswords(team);
  const [group, setGroup] = useState("");
  const [newGroup, setNewGroup] = useState(false);
  const [saved, setSaved] = useState(false);
  const [newPasswords, setNewPasswords] = useState("");
  const selectRef = React.useRef<HTMLSelectElement>(null);

  useEffect(() => {
    if (saved) {
      let index = passwords.findIndex((g) => g.group == group);
      if (index == -1) {
        if (passwords && passwords.length > 0) {
          setGroup(passwords[0].group);
          setNewPasswords(passwords[0].passwords);
        } else {
          setGroup("");
          setNewPasswords("");
          setNewGroup(true);
        }
        return;
      }
      setNewPasswords(passwords[index].passwords);
      if (newGroup) {
        setNewGroup(false);
        if (selectRef.current) {
          selectRef.current.value = group;
        }
      }
      setSaved(false);
    }
    if (group != "") {
      return;
    }
    if (passwordsLoading || passwordsError) return;
    setNewGroup(false);
    if (passwords.length > 0) {
      setGroup(passwords[0].group);
      setNewPasswords(passwords[0].passwords);
      if (selectRef.current) {
        selectRef.current.value = passwords[0].group;
      }
    } else {
      setGroup("");
      setNewPasswords("");
      setNewGroup(true);
    }
  }, [passwords]);

  if (passwordsLoading) return <div>Loading...</div>;
  if (passwordsError) return <div>Error</div>;

  const removeGroup = () => {
    if (group == "") return;
    deletePasswords(group);
    setSaved(true);
  };

  const loadPasswords = (group: string) => {
    if (group == "new") {
      setNewGroup(true);
      setGroup("");
      setNewPasswords("");
      return;
    }
    setNewGroup(false);
    setGroup(group);
    let index = passwords.findIndex((g) => g.group == group);
    if (index == -1) setNewPasswords("");
    setNewPasswords(passwords[index].passwords);
  };

  const onSubmit = () => {
    writePasswords({
      group,
      passwords: {
        passwords: newPasswords,
      },
    });
    setSaved(true);
  };

  return (
    <div className="pl-2">
      <div className="flex">
        <p className="font-light">Set password for group</p>
        <select
          ref={selectRef}
          onChange={(e) => {
            loadPasswords(e.target.value);
          }}
          className="bg-slate-300 mx-1 font-light outline outline-1 rounded px-0.5 outline-slate-400"
        >
          {passwords.map((group) => (
            <option key={group.group} value={group.group}>
              {group.group}
            </option>
          ))}
          <option value="new">New Group:</option>
        </select>
        {newGroup && (
          <input
            placeholder="New Group"
            value={group}
            onChange={(e) => setGroup(e.target.value)}
            className="bg-slate-200 px-0.5 rounded shadow"
          />
        )}
        <input
          onClick={() => onSubmit()}
          type="submit"
          value="Save"
          className="ml-auto bg-slate-200 px-2 rounded shadow"
        />
        {!newGroup && (
          <button
            onClick={() => removeGroup()}
            className="outline outline-1 text-zinc-500 hover:text-red-500 ml-1 rounded-sm px-1"
          >
            Delete Group
          </button>
        )}
      </div>
      <textarea
        className="p-2 h-40 w-40 bg-slate-200 text-sm shadow-inner rounded-md m-1 font-mono resize"
        value={newPasswords}
        spellCheck={false}
        onChange={(e) => {
          setNewPasswords(e.target.value);
        }}
      />
    </div>
  );
};

const TeamDisplay: React.FC<ITeamDisplayProps> = ({ team }) => {
  const { addEnv } = useAddEnv(team.name);
  const { deleteTeam } = useDeleteTeam(team.name);
  const { editTeam } = useEditTeam(team.name);
  const [name, setName] = useState(team.name);

  const onSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const name = formData.get("name") as string;
    const value = formData.get("value") as string;
    addEnv({ name, value });
    // Reset form
    e.currentTarget.reset();
  };

  return (
    <div>
      <div className="flex border-b-2 my-2 border-zinc-400 p-0.5">
        <input
          className="font-medium text-lg flex-grow bg-slate-300 dark:bg-zinc-800 outline-none"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button
          className="ml-auto bg-slate-100 dark:bg-zinc-600 dark:disabled:bg-zinc-800 dark:disabled:bg-opacity-50 rounded-sm px-1 shadow-sm disabled:bg-slate-200 disabled:shadow-none disabled:text-gray-500 hover:shadow-lg active:shadow-none active:bg-slate-200"
          hidden={name == team.name}
          onClick={() => editTeam({ name })}
        >
          Save Team Name
        </button>
        <button
          className="outline outline-1 text-zinc-500 ml-2 hover:text-red-500 rounded-sm px-1"
          onClick={() => deleteTeam()}
        >
          Delete Team
        </button>
      </div>
      <div>
        <h3>Environment</h3>
        {team.env.map((env) => (
          <EnvDisplay env={env} team={team.name} key={env[0]} />
        ))}
        <form className="ml-4 flex my-1 font-mono text-sm" onSubmit={onSubmit}>
          <NaiveShrinker
            className="bg-slate-300 dark:bg-zinc-800 font-medium outline-none"
            name="name"
            placeholder="Name"
          />
          <span>:</span>
          <NaiveShrinker
            className="bg-slate-300 ml-2 dark:bg-zinc-800 font-medium outline-none"
            name="value"
            placeholder="Value"
          />
          <input
            type="submit"
            className="outline outline-1 text-zinc-500 hover:text-green-400 ml-auto rounded-sm px-1"
            value="Add Env"
          />
        </form>
      </div>
      <details>
        <summary>Passwords</summary>
        <PasswordDisplay team={team.name} />
      </details>
    </div>
  );
};

const Teams = () => {
  const { info } = useAdminInfo();
  const { addTeam } = useAddTeam();
  return (
    <div className="p-2 bg-slate-300 m-4 rounded-sm shadow-md pb-1 dark:bg-zinc-800">
      <h2 className="text-2xl text-center font-bold">Teams</h2>
      {info.teams.map((team) => (
        <TeamDisplay team={team} key={team.name} />
      ))}
      <form
        className="flex my-2 p-0.5"
        onSubmit={(e) => {
          e.preventDefault();
          const formData = new FormData(e.currentTarget);
          const name = formData.get("name") as string;
          addTeam({ name });
          // Reset form
          e.currentTarget.reset();
        }}
      >
        <input
          className="font-medium text-lg flex-grow bg-slate-300 dark:bg-zinc-800 outline-none"
          name="name"
          placeholder="Team Name"
        />
        <input
          className="outline outline-1 text-zinc-500 hover:text-green-400 ml-auto rounded-sm px-1"
          value="Add Team"
          type="submit"
        />
      </form>
    </div>
  );
};

export default Teams;
