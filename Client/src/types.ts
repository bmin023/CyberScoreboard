export interface ScoreWrapper {
    teams: [{
        name: string,
        score: number,
        ups: boolean[]
    }],
    services: string[]
}

export interface Score {
    score: number,
    up: boolean,
    history: boolean[]
}

export interface TeamScore {
    services: string[],
    scores: Score[]
}

export interface Service {
    name: string,
    command: string,
    multiplier: number
}

export interface AdminInfo {
    teams: {
        name: string,
        env: [string,string][]
    }[],
    services: Service[],
    active: boolean,
}

export interface TestResult {
    team: string,
    up: boolean,
    message: string,
    error: string
}

export interface EnvPayload {
    name: string,
    value: string
}

export interface TeamPayload {
    name: string,
}

export interface PasswordPayload {
    passwords: string,
}

export interface PasswordBody {
    group: string,
    passwords: string,
}

export interface SaveBody {
    name: string,
    timestamp: number,
}

export interface SaveWrapper {
    saves: SaveBody[],
    autosaves: SaveBody[]
}

export interface SavePayload {
    name: string,
}

export interface InjectDesc {
    uuid: string,
    name: string,
    start: number,
    duration: number,
    completed: boolean,
    file_type?: string[],
}
export interface InjectResponse {
    uuid: string,
    inject_uuid: string,
    late: boolean,
    filename: string,
    upload_time: number,
}
export interface InjectRequest {
    active_injects: InjectDesc[],
    completed_injects: InjectResponse[]
}

export interface InjectData {
    desc: InjectDesc,
    html: string,
    history: InjectResponse[]
}
