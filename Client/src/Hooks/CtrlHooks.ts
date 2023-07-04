import axios from "axios";
import { useMutation, useQuery, useQueryClient } from "react-query";
import {
    AdminInfo,
    CreateInject,
    EnvPayload,
    Inject,
    InjectData,
    InjectRequest,
    PasswordBody,
    PasswordPayload,
    SavePayload,
    SaveWrapper,
    ScoreWrapper,
    Service,
    TeamPayload,
    TeamScore,
    TestResult,
    TimeData,
} from "../types";

const SCORE_REFETCH = 5000;

export const useScore = () => {
    const { data, error, isLoading, dataUpdatedAt } = useQuery(
        "scores",
        async () => {
            const res = await axios.get("/scores");
            return res.data;
        },
        {
            refetchInterval: SCORE_REFETCH,
        }
    );
    return {
        data: data as ScoreWrapper,
        scoreError: error,
        scoreLoading: isLoading,
        scoreUpdatedAt: dataUpdatedAt,
    };
};

export const useTeamScore = (teamName: string | undefined) => {
    if (teamName === undefined) {
        return {
            data: {
                services: [],
                scores: [],
            },
            scoreError: true,
            scoreLoading: true,
            scoreUpdatedAt: 0,
        };
    }
    const { data, error, isLoading, dataUpdatedAt } = useQuery(
        teamName + " scores",
        async () => {
            const res = await axios.get("/scores/" + teamName);
            return res.data;
        },
        {
            refetchInterval: SCORE_REFETCH,
            retry: false,
        }
    );
    return {
        data: data as TeamScore,
        scoreError: error,
        scoreLoading: isLoading,
        scoreUpdatedAt: dataUpdatedAt,
    };
};

export const useAdminInfo = () => {
    const { data, error, isLoading, dataUpdatedAt } = useQuery(
        "adminInfo",
        async () => {
            const res = await axios.get("/admin/config");
            return res.data;
        }
    );
    return {
        info: data as AdminInfo,
        infoError: error,
        infoLoading: isLoading,
        infoUpdatedAt: dataUpdatedAt,
    };
};

export const useEditService = (service: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async (newService: Service) => {
            const res = await axios.post("/admin/service/" + service, newService);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        editService: mutate,
        editServiceLoading: isLoading,
        editServiceError: error,
    };
};

export const useDeleteService = (service: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async () => {
            const res = await axios.delete("/admin/service/" + service);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        deleteService: mutate,
        deleteServiceLoading: isLoading,
        deleteServiceError: error,
    };
};

export const useAddService = () => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async (newService: Service) => {
            const res = await axios.post("/admin/service", newService);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        addService: mutate,
        addServiceLoading: isLoading,
        addServiceError: error,
    };
};

export const useTestService = (service: string) => {
    // make it not trigger on mount and don't refetch
    // wait for user to click test
    const { refetch, data, isLoading, isError, dataUpdatedAt } = useQuery(
        ["test", service],
        async () => {
            const res = await axios.get("/admin/service/" + service);
            return res.data;
        },
        {
            enabled: false,
            refetchOnWindowFocus: false,
            refetchOnMount: false,
            refetchOnReconnect: false,
            refetchInterval: false,
            placeholderData: [],
        }
    );
    return {
        testService: refetch,
        testServiceData: data as TestResult[],
        testServiceUpdatedAt: dataUpdatedAt,
        testServiceLoading: isLoading,
        testServiceError: isError,
    };
};

export const useEditEnv = (team: string, env: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async (newEnv: EnvPayload) => {
            const res = await axios.post(
                "/admin/team/" + team + "/env/" + env,
                newEnv
            );
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        editEnv: mutate,
        editEnvLoading: isLoading,
        editEnvError: error,
    };
};

export const useDeleteEnv = (team: string, env: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async () => {
            const res = await axios.delete("/admin/team/" + team + "/env/" + env);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        deleteEnv: mutate,
        deleteEnvLoading: isLoading,
        deleteEnvError: error,
    };
};

export const useAddEnv = (team: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async (newEnv: EnvPayload) => {
            const res = await axios.post("/admin/team/" + team + "/env", newEnv);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        addEnv: mutate,
        addEnvLoading: isLoading,
        addEnvError: error,
    };
};

export const useEditTeam = (team: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async (newTeam: TeamPayload) => {
            const res = await axios.post("/admin/team/" + team, newTeam);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        editTeam: mutate,
        editTeamLoading: isLoading,
        editTeamError: error,
    };
};

export const useDeleteTeam = (team: string) => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async () => {
            const res = await axios.delete("/admin/team/" + team);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        deleteTeam: mutate,
        deleteTeamLoading: isLoading,
        deleteTeamError: error,
    };
};

export const useAddTeam = () => {
    const queryClient = useQueryClient();
    const { mutate, isLoading, error } = useMutation(
        async (newTeam: TeamPayload) => {
            const res = await axios.post("/admin/team", newTeam);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        addTeam: mutate,
        addTeamLoading: isLoading,
        addTeamError: error,
    };
};

export const useStopGame = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async () => {
            const res = await axios.post("/admin/stop");
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        stopGame: mutate,
    };
};

export const useStartGame = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async () => {
            const res = await axios.post("/admin/start");
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        startGame: mutate,
    };
};

export const useResetScores = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async () => {
            const res = await axios.post("/admin/reset");
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
            },
        }
    );
    return {
        resetScores: mutate,
    };
};

export const useTeamPasswordGroups = (team: string | undefined) => {
    if (team === undefined)
        return { groups: [], groupsLoading: false, groupsError: true };
    const { data, isLoading, error } = useQuery(
        [team, "passgroups"],
        async () => {
            const res = await axios.get("/team/" + team + "/passwords");
            return res.data;
        }
    );
    return {
        groups: data as string[],
        groupsLoading: isLoading,
        groupsError: error,
    };
};

export const useOverwritePasswords = (team: string | undefined) => {
    if (team === undefined) return { overwritePasswords: () => { } };
    const { mutate } = useMutation(
        async ({
            passwords,
            group,
        }: {
            passwords: PasswordPayload;
            group: string;
        }) => {
            const res = await axios.post(
                "/team/" + team + "/passwords/" + group,
                passwords
            );
            return res.data;
        }
    );
    return {
        overwritePasswords: mutate,
    };
};

export const useAdminPasswords = (team: string, onRefetch: (data: PasswordBody[]) => void) => {
    const { data, isLoading, error } = useQuery(
        ["adminPasswords", team],
        async () => {
            const res = await axios.get(`/admin/team/${team}/passwords`);
            return res.data;
        }, {
        onSuccess: (data) => onRefetch(data as PasswordBody[]),
        refetchInterval: Infinity,
    }
    );
    return {
        passwords: data as PasswordBody[],
        passwordsLoading: isLoading,
        passwordsError: error,
    };
};

export const useWritePasswords = (team: string) => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async ({
            passwords,
            group,
        }: {
            passwords: PasswordPayload;
            group: string;
        }) => {
            const res = await axios.post(
                `/admin/team/${team}/passwords/${group}`,
                passwords
            );
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries(["adminPasswords", team]);
            },
        }
    );
    return {
        writePasswords: mutate,
    };
};

export const useDeletePasswords = (team: string) => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async (group: string) => {
            const res = await axios.delete(`/admin/team/${team}/passwords/${group}`);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries(["adminPasswords", team]);
            },
        }
    );
    return {
        deletePasswords: mutate,
    };
};

export const useSaves = () => {
    const { data, isLoading, error } = useQuery("saves", async () => {
        const res = await axios.get("/admin/saves");
        return res.data;
    });
    return {
        saves: data as SaveWrapper,
        savesLoading: isLoading,
        savesError: error,
    };
};

export const useLoadSave = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async (save: SavePayload) => {
            const res = await axios.post("/admin/saves/load", save);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInfo");
                queryClient.invalidateQueries(["adminPasswords"]);
                queryClient.invalidateQueries({ queryKey: ["adminPasswords"] })
            },
        }
    );
    return {
        loadSave: mutate,
    };
};

export const useSaveData = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async (save: SavePayload) => {
            const res = await axios.post("/admin/saves", save);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("saves");
            },
        }
    );
    return {
        saveData: mutate,
    };
};

export const useUploadFile = () => {
    const { mutate } = useMutation(async (formData: FormData) => {
        axios.post('/upload', formData, {
            headers: {
                'Content-Type': 'multipart/form-data'
            }
        })
    });
    return {
        uploadFile: mutate,
    }
}

export const useUploadInject = (team: string | undefined, uuid: string | undefined) => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(async (formData: FormData) => {
        await axios.post(`/team/${team}/injects/${uuid}/upload`, formData, {
            headers: {
                'Content-Type': 'multipart/form-data'
            }
        })
        return;
    }, {
        onSettled: () => {
            queryClient.invalidateQueries(["injects", team, uuid]);
        }
    });
    return {
        uploadInject: mutate,
    }
}

export const useTeamInjects = (team: string | undefined) => {
    if (team === undefined)
        return {
            injects: {
                active_injects: [],
                completed_injects: []
            } as InjectRequest, injectsLoading: false, injectsError: true
        };
    const { data, isLoading, error } = useQuery(
        ["injects", team],
        async () => {
            const res = await axios.get(`/team/${team}/injects`);
            return res.data;
        }
    );
    return {
        injects: data as InjectRequest,
        injectsLoading: isLoading,
        injectsError: error,
    };
}

export const useTeamInject = (team: string | undefined, uuid: string | undefined) => {
    if (team === undefined || uuid === undefined)
        return {
            inject: {
                desc: {
                    uuid: "",
                    name: "",
                    start: 0,
                    duration: 0,
                    completed: false,
                },
                html: "",
            } as InjectData, injectLoading: true, injectError: true
        }
    const { data, isLoading, error } = useQuery(
        ["injects", team, uuid],
        async () => {
            const res = await axios.get(`/team/${team}/injects/${uuid}`);
            return res.data;
        }
    );
    return {
        inject: data as InjectData,
        injectLoading: isLoading,
        injectError: error,
    }
}

export const useTime = (refresh = 60000) => {
    const { data } = useQuery("time", async () => {
        const res = await axios.get("/time");
        return res.data;
    }, {
        refetchInterval: refresh,
        placeholderData: {
            minutes: 0,
            seconds: 0,
            active: true,
        } as TimeData
    });
    return {
        time: data as TimeData,
    };
};

export const useAdminInjects = () => {
    const { data, isLoading, error } = useQuery(
        "adminInjects",
        async () => {
            const res = await axios.get("/admin/injects");
            return res.data;
        }
    );
    return {
        injects: data as Inject[],
        injectsLoading: isLoading,
        injectsError: error,
    };
}

export const useAdminAddInject = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async (inject: CreateInject) => {
            const res = await axios.post("/admin/injects", inject);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInjects");
            },
        }
    );
    return {
        addInject: mutate,
    };
}

export const useAdminEditInject = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async (inject: Inject) => {
            const res = await axios.post(`/admin/injects/${inject.uuid}`, inject);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInjects");
            },
        }
    );
    return {
        editInject: mutate,
    };
}

export const useAdminDeleteInject = () => {
    const queryClient = useQueryClient();
    const { mutate } = useMutation(
        async (uuid: string) => {
            const res = await axios.delete(`/admin/injects/${uuid}`);
            return res.data;
        },
        {
            onSuccess: () => {
                queryClient.invalidateQueries("adminInjects");
            },
        }
    );
    return {
        deleteInject: mutate,
    };
}
